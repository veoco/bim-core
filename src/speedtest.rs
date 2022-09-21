use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use reqwest::Url;
use tokio::sync::{watch, Barrier, Mutex};
use tokio::time::{interval, sleep, timeout};
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;

use crate::requests::{request_http_download, request_http_upload, request_tcp_ping};

pub struct SpeedTest {
    pub provider: String,
    pub download_url: String,
    pub upload_url: String,

    pub ipv6: bool,
    pub thread: u8,
    pub slient: bool,

    address: SocketAddr,

    upload: [u128; 14],
    download: [u128; 14],
    ping: [u128; 10],
    index: [u8; 3],
}

impl SpeedTest {
    pub async fn build(
        provider: String,
        download_url: String,
        upload_url: String,
        ipv6: bool,
        thread: u8,
        slient: bool,
    ) -> Option<SpeedTest> {
        let address = SpeedTest::resolve_ip(&download_url, ipv6)
            .await
            .unwrap_or(None)?;

        Some(SpeedTest {
            provider,
            download_url,
            upload_url,
            ipv6,
            thread,
            slient,
            address,
            upload: [0; 14],
            download: [0; 14],
            ping: [0; 10],
            index: [0; 3],
        })
    }

    pub async fn resolve_ip(url: &str, ipv6: bool) -> Result<Option<SocketAddr>, Box<dyn Error>> {
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::cloudflare(), ResolverOpts::default())?;

        let url = Url::parse(url)?;
        if url.domain().is_none() {
            let addr = url.socket_addrs(|| None).unwrap()[0];
            return Ok(Some(addr));
        }

        let host = url.host_str().unwrap();
        let port = url.port_or_known_default().unwrap();
        if ipv6 {
            let response = resolver.ipv6_lookup(host).await?;
            let address = response.into_iter().next();
            if let Some(addr) = address {
                return Ok(Some(SocketAddr::new(IpAddr::V6(addr), port)));
            }

            return Ok(None);
        } else {
            let response = resolver.ipv4_lookup(host).await?;
            let address = response.into_iter().next();
            if let Some(addr) = address {
                return Ok(Some(SocketAddr::new(IpAddr::V4(addr), port)));
            }

            return Ok(None);
        }
    }

    fn get_speed(&self, array: [u128; 14], i: usize) -> f64 {
        let pos = self.index[i] as usize;
        if pos == 0 {
            return 0.0;
        }
        if pos <= 3 {
            return array[pos - 1] as f64 / pos as f64;
        } else {
            let base = array[2];
            return (array[pos - 1] - base) as f64 / (pos - 3) as f64;
        }
    }

    fn get_upload(&self) -> f64 {
        self.get_speed(self.upload, 0)
    }

    fn set_upload(&mut self, upload: u128) {
        self.upload[self.index[0] as usize] = upload;
        self.index[0] = self.index[0] + 1
    }

    fn get_download(&self) -> f64 {
        self.get_speed(self.download, 1)
    }

    fn set_download(&mut self, download: u128) {
        self.download[self.index[1] as usize] = download;
        self.index[1] = self.index[1] + 1
    }

    fn get_ping(&self) -> f64 {
        let mut ping_min = 1_000_000;
        for ping in self.ping {
            if 0 < ping && ping < ping_min {
                ping_min = ping;
            }
        }
        ping_min as f64
    }

    fn set_ping(&mut self, ping: u128) {
        self.ping[self.index[2] as usize] = ping;
        self.index[2] = self.index[2] + 1
    }

    fn get_jitter(&self) -> f64 {
        let mut ping_min = 1_000_000;
        for ping in self.ping {
            if 0 < ping && ping < ping_min {
                ping_min = ping;
            }
        }
        let mut sum = 0;
        for ping in self.ping {
            if 0 < ping {
                sum += ping - ping_min;
            }
        }
        sum as f64 / 10.0
    }

    pub fn get_result(&self) -> (f64, f64, f64, f64) {
        let upload = self.get_upload() / 125_000.0;
        let download = self.get_download() / 125_000.0;
        let ping = self.get_ping() / 1000.0;
        let jitter = self.get_jitter() / 1000.0;
        (upload, download, ping, jitter)
    }

    async fn ping(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut count = 10;

        while count != 0 {
            let task = request_tcp_ping(&self.address);
            let ping = timeout(Duration::from_micros(1_000_000), task)
                .await
                .unwrap_or(Ok(1_000_000))
                .unwrap_or(1_000_000);
            self.set_ping(ping);
            sleep(Duration::from_millis(500)).await;
            count -= 1;
        }

        if self.get_ping() > 999_999.0 {
            return Ok(false);
        }
        Ok(true)
    }

    async fn download(&mut self) -> Result<bool, Box<dyn Error>> {
        let barrier = Arc::new(Barrier::new((self.thread + 1) as usize));
        let (stop_tx, stop_rx) = watch::channel("run");
        let counter = Arc::new(Mutex::new(0u128));

        let mut url = Url::parse(&self.download_url)?;
        url.set_query(Some("size=25000000&ckSize=1024"));

        for _i in 0..self.thread {
            let url = url.clone();
            let a = self.address.clone();
            let b = barrier.clone();
            let r = stop_rx.clone();
            let c = counter.clone();
            tokio::spawn(async move { request_http_download(url, a, b, r, c).await });
        }

        let mut time_interval = interval(Duration::from_millis(1000));
        let _r = barrier.wait().await;
        time_interval.tick().await;

        for _i in 1..15 {
            time_interval.tick().await;
            let num = {
                let c = counter.lock().await;
                *c
            };
            self.set_download(num);
        }
        stop_tx.send("stop")?;
        sleep(Duration::from_secs(1)).await;

        Ok(true)
    }

    async fn upload(&mut self) -> Result<bool, Box<dyn Error>> {
        let barrier = Arc::new(Barrier::new((self.thread + 1) as usize));
        let (stop_tx, stop_rx) = watch::channel("run");
        let counter = Arc::new(Mutex::new(0u128));

        let url = Url::parse(&self.upload_url)?;

        for _i in 0..self.thread {
            let url = url.clone();
            let a = self.address.clone();
            let b = barrier.clone();
            let r = stop_rx.clone();
            let c = counter.clone();
            tokio::spawn(async move { request_http_upload(url, a, b, r, c).await });
        }

        let mut time_interval = interval(Duration::from_millis(1000));
        let _r = barrier.wait().await;
        time_interval.tick().await;

        for _i in 1..15 {
            time_interval.tick().await;
            let num = {
                let c = counter.lock().await;
                *c
            };
            self.set_upload(num);
        }
        stop_tx.send("stop")?;
        sleep(Duration::from_secs(1)).await;

        Ok(true)
    }

    pub async fn run(&mut self) -> bool {
        let ping = self.ping().await.unwrap_or(false);
        if !ping {
            sleep(Duration::from_secs(1)).await;
            return false;
        } else {
            let _download = self.download().await.unwrap_or(false);
            let _upload = self.upload().await.unwrap_or(false);
        }
        true
    }
}
