use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

use clap::Parser;
use log::info;
use serde_json::Value;
use tokio;

mod utils;
mod requests;
mod speedtest;
use utils::justify_name;
use speedtest::SpeedTest;

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

async fn get_servers(args: &Args) -> Result<Option<HashMap<String, String>>, Box<dyn Error>> {
    let url = format!(
        "https://bench.im/api/search/?type=server&query={}",
        &args.server
    );
    let s = reqwest::get(url)
        .await?
        .json::<Value>()
        .await?
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
    r.insert(
        String::from("dl"),
        detail.get("dl").unwrap().as_str().unwrap().to_string(),
    );
    r.insert(
        String::from("ul"),
        detail.get("ul").unwrap().as_str().unwrap().to_string(),
    );

    Ok(Some(r))
}

async fn run_once(args: Args) -> (f64, f64, f64, f64) {
    let location = get_servers(&args).await;
    let location = match location {
        Ok(Some(l))=> {l}
        _ => return (0.0, 0.0, 0.0, 0.0)
    };

    let provider = location.get("provider").unwrap().clone();

    let ipv6 = location.get("ipv6").unwrap().clone();
    let ipv6 = if ipv6 == "false" { false } else { true };
    if args.ipv6 {
        if !ipv6 {
            return (0.0, 0.0, 0.0, 0.0)
        }
    }

    let download_url = location.get("dl").unwrap().clone();
    let upload_url = location.get("ul").unwrap().clone();

    let client = SpeedTest::build(
        provider,
        download_url,
        upload_url,
        if args.ipv6 && ipv6 { true } else { false },
        args.thread,
        false,
    )
    .await;

    if let Some(mut c) = client {
        let res = c.run().await;
        if res {
            let r = c.get_result();
            return r
        }
    }
    return (0.0, 0.0, 0.0, 0.0)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let args_name = args.name.clone();
    if let Some(name) = args_name {
        print!("{}", justify_name(&name));
        return;
    }

    env_logger::init();

    info!("Enter oneshot mode");
    let (download, upload, ping, jitter) = run_once(args).await;
    println!("{:.1},{:.1},{:.1},{:.1}", download, upload, ping, jitter);
}
