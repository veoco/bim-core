use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use std::thread;
use std::time::Duration;

use log::debug;
use url::Url;

use crate::requests::{request_http_download, request_http_upload, request_tcp_ping, request_https_download, request_https_upload};

pub struct SpeedTest {
    pub provider: String,
    pub download_url: Url,
    pub upload_url: Url,

    pub ipv6: bool,
    pub connection_close: bool,

    address: SocketAddr,

    upload: f64,
    download: f64,
    ping: f64,
    jitter: f64,
}

impl SpeedTest {
    pub fn build(
        provider: String,
        download_url: String,
        upload_url: String,
        ipv6: bool,
        connection_close: bool,
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

        Some(SpeedTest {
            provider,
            download_url,
            upload_url,
            ipv6,
            connection_close,
            address,
            upload: 0.0,
            download: 0.0,
            ping: 0.0,
            jitter: 0.0,
        })
    }

    pub fn get_result(&self) -> (f64, f64, f64, f64) {
        let upload = self.upload;
        let download = self.download;
        let ping = self.ping;
        let jitter = self.jitter;
        (upload, download, ping, jitter)
    }

    fn ping(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut count = 0;
        let mut pings = [0u128; 10];
        let mut ping_min = 1_000_000;

        while count < 10 {
            let ping = request_tcp_ping(&self.address).unwrap_or(1_000_000);
            pings[count] = ping;
            if ping < ping_min {
                ping_min = ping;
            }
            thread::sleep(Duration::from_millis(500));
            count += 1;
        }
        if ping_min > 999_999 {
            return Ok(false);
        }

        let mut jitter_all = 0;
        for p in pings {
            jitter_all += p - ping_min;
        }

        self.ping = ping_min as f64 / 1_000.0;
        self.jitter = jitter_all as f64 / 1_000_0.0;

        debug!("Ping {} ms", self.ping);
        debug!("Jitter {} ms", self.jitter);

        Ok(true)
    }

    fn download(&mut self) -> Result<bool, Box<dyn Error>> {
        let url = self.download_url.clone();
        let address = self.address.clone();

        let res;
        if url.scheme() == "https" {
            res = request_https_download(address, url, self.connection_close)?;
        } else {
            res = request_http_download(address, url, self.connection_close)?;
        }
        self.download = res;

        Ok(true)
    }

    fn upload(&mut self) -> Result<bool, Box<dyn Error>> {
        let url = self.upload_url.clone();
        let address = self.address.clone();

        let res;
        if url.scheme() == "https" {
            res = request_https_upload(address, url, self.connection_close)?;
        } else {
            res = request_http_upload(address, url, self.connection_close)?;
        }
        self.upload = res;

        Ok(true)
    }

    pub fn run(&mut self) -> bool {
        let ping = self.ping().unwrap_or(false);
        if !ping {
            return false;
        } else {
            thread::sleep(Duration::from_secs(3));
            let _upload = self.upload();
            thread::sleep(Duration::from_secs(3));
            let _download = self.download();
        }
        true
    }
}
