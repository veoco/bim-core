use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{
    atomic::{AtomicU64, AtomicU8, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(debug_assertions)]
use log::debug;

use url::Url;

use crate::clients::base::{
    check_running, make_connection, wait_ready, wait_stop, wait_sync, Client,
};
use crate::utils::SpeedTestResult;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::SystemTime;

pub struct HTTPClient {
    download_url: Url,
    upload_url: Url,
    multi_thread: bool,

    address: SocketAddr,

    upload: f64,
    upload_status: String,
    download: f64,
    download_status: String,
    latency: f64,
    jitter: f64,
}

impl HTTPClient {
    pub fn build(
        download_url: String,
        upload_url: String,
        ipv6: bool,
        multi_thread: bool,
    ) -> Option<Self> {
        let download_url = Url::parse(&download_url).ok()?;
        let upload_url = Url::parse(&upload_url).ok()?;

        let host = download_url.host_str()?.to_owned();
        let port = download_url.port_or_known_default().to_owned()?;

        let host_port = format!("{host}:{port}");
        let addresses = host_port.to_socket_addrs().ok()?;

        let address = addresses
            .into_iter()
            .find(|addr| (addr.is_ipv6() && ipv6) || (addr.is_ipv4() && !ipv6))?;

        #[cfg(debug_assertions)]
        debug!("IP address {address}");

        let r = "取消".to_owned();
        Some(Self {
            download_url,
            upload_url,
            multi_thread,
            address,
            upload: 0.0,
            upload_status: r.clone(),
            download: 0.0,
            download_status: r.clone(),
            latency: 0.0,
            jitter: 0.0,
        })
    }

    fn run_load(&mut self, load: u8) -> Result<bool, Box<dyn Error>> {
        let url = match load {
            0 => self.upload_url.clone(),
            _ => self.download_url.clone(),
        };
        let threads = if self.multi_thread { 8 } else { 1 };
        let counter = Arc::new(AtomicU64::new(0));
        let flag = Arc::new(AtomicU8::new(threads));

        for _ in 0..threads {
            let a = self.address.clone();
            let u = url.clone();
            let c = counter.clone();
            let f = flag.clone();

            thread::spawn(move || {
                let _ = match load {
                    0 => request_http_upload(a, u, c, f),
                    _ => request_http_download(a, u, c, f),
                };
            });
            thread::sleep(Duration::from_millis(250));
        }

        let mut time_passed = 0;
        let mut results = [(0, 0); 28];
        let mut index = 0;

        wait_sync(&flag);

        let now = Instant::now();
        while time_passed < 14_000_000 {
            thread::sleep(Duration::from_millis(500));
            time_passed = now.elapsed().as_micros();

            results[index] = (counter.load(Ordering::Relaxed), time_passed);
            index += 1;
        }

        flag.store(threads, Ordering::SeqCst);
        wait_sync(&flag);

        #[cfg(debug_assertions)]
        debug!("Results {results:?}");

        let speed = {
            let (c18, t18) = results[17];
            let (c28, t28) = results[index - 1];

            ((c28 - c18) * 8) as f64 / (t28 - t18) as f64
        };

        let status = {
            let mut stop = 0;
            let mut last = 0;
            for (num, _) in results {
                if num == last {
                    stop += 1;
                }
                last = num;
            }

            if stop < 6 {
                "正常"
            } else {
                "断流"
            }
        }
        .to_string();

        match load {
            0 => {
                self.upload = speed;
                self.upload_status = status;
            }
            _ => {
                self.download = speed;
                self.download_status = status;
            }
        }

        Ok(true)
    }
}

impl Client for HTTPClient {
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

fn request_tcp_ping(address: &SocketAddr) -> u128 {
    let now = Instant::now();
    let r = TcpStream::connect_timeout(&address, Duration::from_micros(1_000_000));
    let used = now.elapsed().as_micros();
    match r {
        Ok(_) => used,
        Err(_e) => {
            #[cfg(debug_assertions)]
            debug!("Ping {_e}");

            0
        }
    }
}

fn request_http_download(
    address: SocketAddr,
    url: Url,
    counter: Arc<AtomicU64>,
    flag: Arc<AtomicU8>,
) {
    let chunk_count = 50;
    let data_size = chunk_count * 1024 * 1024 as u64;
    let mut data_counter;
    let mut buffer = [0; 65536];

    let host_port = format!(
        "{}:{}",
        url.host_str().unwrap(),
        url.port_or_known_default().unwrap()
    );
    let path_str = url.path();

    let mut stream = match make_connection(&address, &url) {
        Ok(s) => s,
        Err(_) => {
            wait_ready(&flag);
            wait_stop(&flag);
            return;
        }
    };

    let rand_time = flag.load(Ordering::Relaxed) * 30;
    wait_ready(&flag);
    thread::sleep(Duration::from_millis(rand_time as u64));

    'request: while check_running(&flag) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let path_query = format!(
            "{}?cors=true&r={}&ckSize={}&size={}",
            path_str, now, chunk_count, data_size
        );

        #[cfg(debug_assertions)]
        debug!("Download {path_query}");

        let request_head = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: bim/1.0\r\n\r\n",
            path_query, host_port,
        )
        .into_bytes();

        match stream.write_all(&request_head) {
            Ok(_) => {
                if let Ok(size) = stream.read(&mut buffer) {
                    #[cfg(debug_assertions)]
                    debug!("Download Status: {size}");

                    if size > 0 {
                        let count = size as u64;
                        data_counter = count;
                        counter.fetch_add(count, Ordering::Relaxed);
                    } else {
                        break 'request;
                    }
                } else {
                    break 'request;
                }
            }
            Err(_e) => {
                #[cfg(debug_assertions)]
                debug!("Download Error: {}", _e);

                break 'request;
            }
        }

        while data_counter < data_size && check_running(&flag) {
            match stream.read(&mut buffer) {
                Ok(size) => {
                    let count = size as u64;
                    data_counter += count;
                    counter.fetch_add(count, Ordering::Relaxed);
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    debug!("Download Error: {}", _e);

                    break 'request;
                }
            }
        }
    }
    wait_stop(&flag);
}

fn request_http_upload(
    address: SocketAddr,
    url: Url,
    counter: Arc<AtomicU64>,
    flag: Arc<AtomicU8>,
) {
    let chunk_count = 50;
    let data_size = chunk_count * 1024 * 1024 as u64;
    let mut data_counter;

    let host_port = format!(
        "{}:{}",
        url.host_str().unwrap(),
        url.port_or_known_default().unwrap()
    );
    let url_path = url.path();
    let request_chunk = "0123456789AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz-="
        .repeat(1024)
        .into_bytes();

    let mut stream = match make_connection(&address, &url) {
        Ok(s) => s,
        Err(_) => {
            wait_ready(&flag);
            wait_stop(&flag);
            return;
        }
    };

    let rand_time = flag.load(Ordering::Relaxed) * 30;
    wait_ready(&flag);
    thread::sleep(Duration::from_millis(rand_time as u64));

    'request: while check_running(&flag) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let path_query = format!("{}?r={}", url_path, now);

        #[cfg(debug_assertions)]
        debug!("Upload {path_query} size {data_size}");

        let request_head = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: bim/1.0\r\nContent-Length: {}\r\n\r\n",
            path_query, host_port, data_size
        )
        .into_bytes();

        match stream.write_all(&request_head) {
            Ok(_) => {
                let length = request_head.len() as u64;
                data_counter = length;
                counter.fetch_add(length, Ordering::SeqCst);
            }
            Err(_e) => {
                #[cfg(debug_assertions)]
                debug!("Upload Error: {}", _e);

                break 'request;
            }
        }

        while data_counter < data_size && check_running(&flag) {
            match stream.write(&request_chunk) {
                Ok(size) => {
                    let count = size as u64;
                    data_counter += size as u64;
                    counter.fetch_add(count, Ordering::Relaxed);
                }
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    debug!("Upload Error: {}", _e);

                    break 'request;
                }
            }
        }
    }
    wait_stop(&flag);
}
