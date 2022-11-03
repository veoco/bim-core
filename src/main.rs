use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

use clap::Parser;
use log::debug;
use serde_json::Value;

mod requests;
mod speedtest;
mod utils;
use speedtest::SpeedTest;
use utils::justify_name;

/// Simple program to test network
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    server: u32,
    /// Enable IPv6 only test
    #[clap(short = '6', long, action)]
    ipv6: bool,
    /// Number of thread
    #[clap(short, long, value_parser, default_value_t = 1)]
    thread: u8,
    /// Name justify
    #[clap(short, long, value_parser)]
    name: Option<String>,
}

fn get_server(args: &Args) -> Result<Option<HashMap<String, String>>, Box<dyn Error>> {
    let url = format!(
        "https://bench.im/api/search/?type=server&query={}",
        &args.server
    );
    debug!("Start get server {}", &args.server);

    let s = minreq::get(url)
        .send()?
        .json::<Value>()?
        .get("results")
        .unwrap()
        .get(0)
        .unwrap()
        .clone();

    let provider = s.get("provider").unwrap().as_str().unwrap().to_string();
    let detail = s.get("detail").unwrap();
    let mut r = HashMap::new();

    r.insert(String::from("provider"), provider.clone());
    r.insert(
        String::from("ipv6"),
        detail.get("ipv6").unwrap().to_string(),
    );

    let dl = detail.get("dl").unwrap().as_str().unwrap().to_string();
    r.insert(
        String::from("dl"),
        detail.get("dl").unwrap().as_str().unwrap().to_string(),
    );
    r.insert(
        String::from("ul"),
        detail.get("ul").unwrap().as_str().unwrap().to_string(),
    );

    if dl.contains("10000gd.tech") {
        r.insert(
            String::from("connection_close"),
            String::from("true"),
        );
    }else{
        r.insert(
            String::from("connection_close"),
            String::from("false"),
        );
    }

    debug!("Got server {}", dl);
    Ok(Some(r))
}

fn run(args: Args) -> (f64, f64, f64, f64) {
    let location = get_server(&args);
    let location = match location {
        Ok(Some(l)) => l,
        _ => return (0.0, 0.0, 0.0, 0.0),
    };

    let provider = location.get("provider").unwrap().clone();

    let ipv6 = location.get("ipv6").unwrap().clone();
    let ipv6 = if ipv6 == "false" { false } else { true };
    if args.ipv6 {
        if !ipv6 {
            return (0.0, 0.0, 0.0, 0.0);
        }
    }

    let connection_close = location.get("connection_close").unwrap().clone();
    let connection_close = if connection_close == "false" { false } else { true };

    let download_url = location.get("dl").unwrap().clone();
    let upload_url = location.get("ul").unwrap().clone();

    let client = SpeedTest::build(
        provider,
        download_url,
        upload_url,
        if args.ipv6 && ipv6 { true } else { false },
        connection_close,
    );

    if let Some(mut c) = client {
        let res = c.run();
        if res {
            let r = c.get_result();
            return r;
        }
    }
    return (0.0, 0.0, 0.0, 0.0);
}

fn main() {
    let args = Args::parse();

    let args_name = args.name.clone();
    if let Some(name) = args_name {
        print!("{}", justify_name(&name));
        return;
    }

    env_logger::init();

    let (download, upload, ping, jitter) = run(args);

    println!("{:.1},{:.1},{:.1},{:.1}", download, upload, ping, jitter);
}
