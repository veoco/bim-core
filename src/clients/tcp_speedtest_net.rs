use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(debug_assertions)]
use log::debug;

use url::Url;

use crate::clients::base::{get_address, make_connection, request_tcp_ping, Client, LoadCounter};
use crate::utils::SpeedTestResult;

use std::io::{Read, Write};

pub struct SpeedtestNetTcpClient {
    threads: u8,

    address: SocketAddr,

    upload: f64,
    upload_status: String,
    download: f64,
    download_status: String,
    latency: f64,
    jitter: f64,
}

impl SpeedtestNetTcpClient {
    pub fn build(url: String, ipv6: bool, threads: u8) -> Option<Box<dyn Client>> {
        let url = Url::parse(&url).ok()?;

        let address = get_address(&url, ipv6)?;

        #[cfg(debug_assertions)]
        debug!("IP address {address}");

        let r = "取消".to_owned();
        Some(Box::new(Self {
            threads,
            address,
            upload: 0.0,
            upload_status: r.clone(),
            download: 0.0,
            download_status: r.clone(),
            latency: 0.0,
            jitter: 0.0,
        }))
    }

    fn run_load(&mut self, load: u8) -> Result<bool, Box<dyn Error>> {
        let counter = Arc::new(LoadCounter::new(self.threads));
        let mut tasks = vec![];

        for _ in 0..self.threads {
            let a = self.address.clone();
            let c = counter.clone();

            let task = thread::spawn(move || {
                match load {
                    0 => request_tcp_upload(a, c),
                    _ => request_tcp_download(a, c),
                };
            });
            tasks.push(task);
            thread::sleep(Duration::from_millis(250));
        }

        let mut time_passed = 0;

        counter.wait();

        let now = Instant::now();
        while time_passed < 14_000_000 {
            thread::sleep(Duration::from_millis(500));
            time_passed = now.elapsed().as_micros();

            counter.count(time_passed);
        }

        counter.end();
        for task in tasks {
            let _ = task.join();
        }

        match load {
            0 => {
                self.upload = counter.speed();
                self.upload_status = counter.status();
            }
            _ => {
                self.download = counter.speed();
                self.download_status = counter.status();
            }
        }

        Ok(true)
    }
}

impl Client for SpeedtestNetTcpClient {
    fn ping(&mut self) -> bool {
        let mut count = 0;
        let mut pings = [0u128; 6];
        let mut ping_min = 10000000;

        while count < 6 {
            let ping = request_tcp_ping(&self.address);
            if ping > 0 {
                if ping < ping_min {
                    ping_min = ping
                }
                pings[count] = ping;
            }
            thread::sleep(Duration::from_millis(1000));
            count += 1;
        }

        if pings == [0, 0, 0, 0, 0, 0] {
            self.latency = 0.0;
            self.jitter = 0.0;
            return false;
        }

        let mut jitter_all = 0;
        for p in pings {
            if p > 0 {
                jitter_all += p - ping_min;
            }
        }

        self.latency = ping_min as f64 / 1_000.0;
        self.jitter = jitter_all as f64 / 5_000.0;

        #[cfg(debug_assertions)]
        debug!("Ping {} ms", self.latency);

        #[cfg(debug_assertions)]
        debug!("Jitter {} ms", self.jitter);

        true
    }

    fn download(&mut self) -> bool {
        match self.run_load(1) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn upload(&mut self) -> bool {
        match self.run_load(0) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn result(&self) -> SpeedTestResult {
        SpeedTestResult::build(
            self.upload,
            self.upload_status.clone(),
            self.download,
            self.download_status.clone(),
            self.latency,
            self.jitter,
        )
    }
}

fn request_tcp_download(address: SocketAddr, counter: Arc<LoadCounter>) {
    let data_size = 15 * 1024 * 1024 * 1024 as u128;
    let mut buffer = [0; 65536];

    let url = Url::parse("http://bench.im").unwrap();
    let mut stream = match make_connection(&address, &url) {
        Ok(s) => s,
        Err(_) => {
            counter.wait();
            return;
        }
    };

    counter.wait();

    #[cfg(debug_assertions)]
    debug!("Download Start");

    let request = format!("DOWNLOAD {data_size}\n").into_bytes();
    match stream.write_all(&request) {
        Ok(_) => {
            if let Ok(size) = stream.read(&mut buffer) {
                #[cfg(debug_assertions)]
                debug!("Download Status: {size}");

                if size == 0 {
                    #[cfg(debug_assertions)]
                    debug!("Download Error: Start failed");
                    return;
                }

                let count = size as u64;
                counter.increase(count);
            } else {
                return;
            }
        }
        Err(_e) => {
            #[cfg(debug_assertions)]
            debug!("Download Error: {}", _e);
            return;
        }
    }

    while !counter.is_end() {
        match stream.read(&mut buffer) {
            Ok(size) => {
                let count = size as u64;
                counter.increase(count);
            }
            Err(_e) => {
                #[cfg(debug_assertions)]
                debug!("Download Error: {}", _e);
                return;
            }
        }
    }
}

fn request_tcp_upload(address: SocketAddr, counter: Arc<LoadCounter>) {
    let data_size = 15 * 1024 * 1024 * 1024 as u128;
    let request_chunk = "0123456789AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz-="
        .repeat(1024)
        .into_bytes();

    let url = Url::parse("http://bench.im").unwrap();

    let mut stream = match make_connection(&address, &url) {
        Ok(s) => s,
        Err(_) => {
            counter.wait();
            return;
        }
    };

    counter.wait();

    #[cfg(debug_assertions)]
    debug!("Upload Start");

    let request = format!("UPLOAD {data_size} 0\n").into_bytes();
    match stream.write_all(&request) {
        Ok(_) => {
            let count = request.len() as u64;
            counter.increase(count);
        }
        Err(_e) => {
            #[cfg(debug_assertions)]
            debug!("Upload Error: {}", _e);
            return;
        }
    }

    while !counter.is_end() {
        match stream.write(&request_chunk) {
            Ok(size) => {
                let count = size as u64;
                counter.increase(count);
            }
            Err(_e) => {
                #[cfg(debug_assertions)]
                debug!("Upload Error: {}", _e);
                return;
            }
        }
    }
}
