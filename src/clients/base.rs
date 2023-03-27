use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(debug_assertions)]
use log::debug;

use rustls::{OwnedTrustAnchor, RootCertStore};
use url::Url;

use crate::utils::SpeedTestResult;

pub trait GenericStream: Read + Write {}

impl<T: Read + Write> GenericStream for T {}

pub fn make_connection(
    address: &SocketAddr,
    url: &Url,
) -> Result<Box<dyn GenericStream>, String> {
    let ssl = if url.scheme() == "https" { true } else { false };
    let mut retry = 3;

    while retry > 0 {
        if let Ok(stream) = TcpStream::connect_timeout(&address, Duration::from_micros(3_000_000)) {
            #[cfg(debug_assertions)]
            debug!("TCP connected");

            let _r = stream.set_write_timeout(Some(Duration::from_secs(3)));
            let _r = stream.set_read_timeout(Some(Duration::from_secs(3)));
            if !ssl {
                return Ok(Box::new(stream));
            }

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
            let tls = rustls::StreamOwned::new(conn, stream);

            #[cfg(debug_assertions)]
            debug!("SSL connected");

            return Ok(Box::new(tls));
        }

        retry -= 1;
    }
    return Err(String::from("连接失败"));
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
