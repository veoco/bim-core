use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::thread;

#[cfg(debug_assertions)]
use log::debug;

use tiny_http::Method;

use crate::servers::Server;

pub struct HTTPServer {
    address: String,
}

impl HTTPServer {
    pub fn build(address: String) -> Option<Self> {
        let _ = match address.to_socket_addrs() {
            Ok(_) => {}
            Err(_) => return None,
        };

        return Some(Self { address });
    }
}

impl Server for HTTPServer {
    fn run(&mut self) -> bool {
        let server = match tiny_http::Server::http(&self.address) {
            Ok(s) => Arc::new(s),
            Err(_e) => {
                #[cfg(debug_assertions)]
                debug!("Start Failed {_e}");

                return false;
            }
        };

        let data = Arc::new(
            "0123456789AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz-="
                .repeat(1024)
                .into_bytes(),
        );

        let mut guards = Vec::with_capacity(8);

        for _ in 0..8 {
            let server = server.clone();
            let data = data.clone();

            let guard = thread::spawn(move || loop {
                let request = server.recv().unwrap();
                let method = request.method().clone();
                let mut writer = request.into_writer();
                let data = data.as_ref();

                match method {
                    Method::Get => {
                        let mut counter = 0;
                        let head = "HTTP/1.1 200\r\n\r\n".as_bytes();
                        let _ = writer.write_all(head);

                        while counter < 50 * 1024 * 1024 {
                            let _ = writer.write_all(data);
                            counter += 65536;
                        }
                    }
                    Method::Post => {
                        let head = "HTTP/1.1 200\r\n\r\n".as_bytes();
                        let _ = writer.write_all(head);
                    }
                    _ => {
                        let head = "HTTP/1.1 500\r\n\r\n".as_bytes();
                        let _ = writer.write_all(head);
                    }
                }
            });

            guards.push(guard);
        }

        for p in guards {
            let _ = p.join();
        }
        true
    }
}
