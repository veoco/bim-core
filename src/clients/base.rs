use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::{Arc, Barrier, RwLock};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(debug_assertions)]
use log::debug;

use rustls::{OwnedTrustAnchor, RootCertStore};
use url::Url;

use crate::utils::SpeedTestResult;

pub trait GenericStream: Read + Write {}

impl<T: Read + Write> GenericStream for T {}

pub fn get_address(url: &Url, ipv6: bool) -> Option<SocketAddr> {
    let host = url.host_str()?;
    let port = url.port_or_known_default()?;

    let host_port = format!("{host}:{port}");
    let addresses = host_port.to_socket_addrs().ok()?;

    addresses
        .into_iter()
        .find(|addr| (addr.is_ipv6() && ipv6) || (addr.is_ipv4() && !ipv6))
}

pub fn make_connection(address: &SocketAddr, url: &Url) -> Result<Box<dyn GenericStream>, String> {
    let ssl = if url.scheme() == "https" { true } else { false };
    let mut retry = 3;

    let mut root_store = RootCertStore::empty();
    root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(
        |ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        },
    ));

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let server_name = url.host_str().unwrap().try_into().unwrap();
    let conn = rustls::ClientConnection::new(Arc::new(config), server_name).unwrap();

    while retry > 0 {
        if let Ok(stream) = TcpStream::connect_timeout(&address, Duration::from_micros(1_000_000)) {
            #[cfg(debug_assertions)]
            debug!("TCP connected");

            let _r = stream.set_write_timeout(Some(Duration::from_secs(3)));
            let _r = stream.set_read_timeout(Some(Duration::from_secs(3)));
            if !ssl {
                return Ok(Box::new(stream));
            }

            let tls = rustls::StreamOwned::new(conn, stream);

            #[cfg(debug_assertions)]
            debug!("SSL connected");

            return Ok(Box::new(tls));
        }

        retry -= 1;
    }
    
    Err(String::from("连接失败"))
}

pub fn request_tcp_ping(address: &SocketAddr) -> u128 {
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

pub struct LoadCounter {
    counter: RwLock<u64>,
    stater: Barrier,
    ender: RwLock<bool>,
    results: RwLock<Vec<(u64, u128)>>,
}

impl LoadCounter {
    pub fn new(threads: u8) -> Self {
        Self {
            counter: RwLock::new(0),
            stater: Barrier::new((threads + 1) as usize),
            ender: RwLock::new(false),
            results: RwLock::new(vec![]),
        }
    }

    pub fn wait(&self) {
        self.stater.wait();
    }

    pub fn end(&self) {
        let mut e = self.ender.write().unwrap();
        *e = true;
    }

    pub fn is_end(&self) -> bool {
        let e = self.ender.read().unwrap();
        *e
    }

    pub fn increase(&self, count: u64) {
        let mut c = self.counter.write().unwrap();
        *c += count;
    }

    pub fn count(&self, time_passed: u128) {
        let c = { *self.counter.read().unwrap() };

        let mut results = self.results.write().unwrap();
        results.push((c, time_passed));
    }

    pub fn speed(&self) -> f64 {
        let (c18, t18) = self.results.read().unwrap()[17];
        let (c28, t28) = self.results.read().unwrap()[27];

        ((c28 - c18) * 8) as f64 / (t28 - t18) as f64
    }

    pub fn status(&self) -> String {
        let mut stop = 0;
        let mut last = 0;
        let results = self.results.read().unwrap().to_vec();

        #[cfg(debug_assertions)]
        debug!("Results {results:?}");

        for (num, _) in results {
            if num == last {
                stop += 1;
            }
            last = num;
        }

        if stop < 6 {
            String::from("正常")
        } else {
            String::from("断流")
        }
    }
}

pub trait Client {
    fn result(&self) -> SpeedTestResult;

    fn ping(&mut self) -> bool;

    fn upload(&mut self) -> bool;

    fn download(&mut self) -> bool;

    fn run(&mut self) -> bool {
        let r = self.ping();
        if r {
            thread::sleep(Duration::from_secs(2));
            self.upload();
            thread::sleep(Duration::from_secs(3));
            self.download();
            return true;
        }
        false
    }
}
