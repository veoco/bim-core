use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use log::debug;
use url::Url;

use crate::requests::{request_http_download, request_http_upload, request_tcp_ping};
use crate::utils::justify_name;

pub struct SpeedTest {
    pub download_url: Url,
    pub upload_url: Url,
    pub ipv6: bool,
    pub connection_close: bool,
    pub multi_thread: bool,

    address: SocketAddr,

    pub result: (String, String, String, String),
}

impl SpeedTest {
    pub fn build(
        download_url: String,
        upload_url: String,
        ipv6: bool,
        connection_close: bool,
        multi_thread: bool,
    ) -> Option<SpeedTest> {
        let download_url = match Url::parse(&download_url) {
            Ok(u) => u,
            Err(_) => return None,
        };
        let upload_url = match Url::parse(&upload_url) {
            Ok(u) => u,
            Err(_) => return None,
        };

        let host = match download_url.host_str() {
            Some(h) => h,
            None => return None,
        };
        let port = match download_url.port_or_known_default() {
            Some(p) => p,
            None => return None,
        };

        let host_port = format!("{host}:{port}");
        let addresses = match host_port.to_socket_addrs() {
            Ok(addrs) => addrs,
            Err(_) => return None,
        };

        let mut address = None;
        for addr in addresses {
            if (addr.is_ipv6() && ipv6) || (addr.is_ipv4() && !ipv6) {
                address = Some(addr);
            }
        }

        let address = match address {
            Some(addr) => addr,
            None => return None,
        };
        debug!("IP address {address}");

        let r = String::from("未启动");
        Some(SpeedTest {
            download_url,
            upload_url,
            ipv6,
            connection_close,
            multi_thread,
            address,
            result: (r.clone(), r.clone(), r.clone(), r.clone()),
        })
    }

    fn ping(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut count = 0;
        let mut pings = [0u128; 6];
        let mut ping_min = 1_000_000;

        while count < 6 {
            let ping = request_tcp_ping(&self.address).unwrap_or(1_000_000);
            pings[count] = ping;
            if ping < ping_min {
                ping_min = ping;
            }
            thread::sleep(Duration::from_millis(500));
            count += 1;
        }
        if ping_min > 999_999 {
            self.result.2 = String::from("失败");
            self.result.3 = String::from("失败");
            return Ok(false);
        }

        let mut jitter_all = 0;
        for p in pings {
            jitter_all += p - ping_min;
        }

        self.result.2 = format!("{:.1}", ping_min as f64 / 1_000.0);
        self.result.3 = format!("{:.1}", jitter_all as f64 / 5_000.0);

        debug!("Ping {} ms", self.result.2);
        debug!("Jitter {} ms", self.result.3);

        Ok(true)
    }

    fn run_load(&mut self, load: u8) -> Result<bool, Box<dyn Error>> {
        let url = match load {
            0 => self.upload_url.clone(),
            _ => self.download_url.clone(),
        };
        let ssl = if url.scheme() == "https" { true } else { false };
        let threads = if self.multi_thread { 8 } else { 1 };
        let counter = Arc::new(Mutex::new(0u128));
        let barrier = Arc::new(Barrier::new(threads + 1));

        for _ in 0..threads {
            let a = self.address.clone();
            let u = url.clone();
            let c = self.connection_close.clone();
            let s = ssl.clone();
            let ct = counter.clone();
            let b = barrier.clone();

            thread::spawn(move || {
                let _ = match load {
                    0 => request_http_upload(a, u, c, s, ct, b),
                    _ => request_http_download(a, u, c, s, ct, b),
                };
            });
        }

        let mut last = 0;
        let mut last_time = 0;
        let mut time_passed = 0;
        let mut results = [0.0; 28];
        let mut index = 0;
        let mut wait = 6;

        barrier.wait();
        let now = Instant::now();
        while time_passed < 14_000_000 {
            thread::sleep(Duration::from_millis(500));
            time_passed = now.elapsed().as_micros();
            let time_used = time_passed - last_time;
            let current = {
                let ct = counter.lock().unwrap();
                *ct
            };
            if last == current {
                wait -= 1;
            }
            let speed = ((current - last) * 8) as f64 / time_used as f64;
            debug!("Transfered {current} bytes in {time_passed} us, speed {speed}");
            results[index] = speed;
            index += 1;
            last = current;
            last_time = time_passed;
        }

        let mut all = 0.0;
        for i in index - 20..index {
            all += results[i];
        }
        let final_speed = all / 20.0;

        let res = if wait <= 0 {
            if last == 0 {
                format!("失败")
            }else {
                format!("断流")
            }
        } else {
            format!("{:.1}", final_speed)
        };

        match load {
            0 => self.result.0 = res,
            _ => self.result.1 = res,
        }

        Ok(true)
    }

    fn download(&mut self) -> Result<bool, Box<dyn Error>> {
        let _ = self.run_load(1)?;
        Ok(true)
    }

    fn upload(&mut self) -> Result<bool, Box<dyn Error>> {
        let _ = self.run_load(0)?;
        Ok(true)
    }

    pub fn get_result(&self) -> (String, String, String, String) {
        let upload = justify_name(&self.result.0, 11);
        let download = justify_name(&self.result.1, 11);
        let ping = justify_name(&self.result.2, 9);
        let jitter = justify_name(&self.result.3, 7);
        (upload, download, ping, jitter)
    }

    pub fn run(&mut self) -> bool {
        let ping = self.ping().unwrap_or(false);
        if !ping {
            return false;
        } else {
            thread::sleep(Duration::from_secs(1));
            let _upload = self.upload();
            thread::sleep(Duration::from_secs(1));
            let _download = self.download();
            thread::sleep(Duration::from_secs(1));
        }
        true
    }
}
