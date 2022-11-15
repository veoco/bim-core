use std::fmt::Debug;

use clap::Parser;

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
    download_url: String,
    #[clap(value_parser)]
    upload_url: String,
    /// Enable IPv6 only test
    #[clap(short = '6', long, action, default_value_t = false)]
    ipv6: bool,
    /// Enable connection close mode
    #[clap(short, long, action, default_value_t = false)]
    connection_close: bool,
    /// Enable multi thread mode
    #[clap(short, long, action, default_value_t = false)]
    multi_thread: bool,
    /// Name justify
    #[clap(short, long, value_parser)]
    name: Option<String>,
}

fn run(args: Args) -> (String, String, String, String) {
    let client = SpeedTest::build(
        args.download_url,
        args.upload_url,
        args.ipv6,
        args.connection_close,
        args.multi_thread,
    );

    if let Some(mut c) = client {
        let res = c.run();
        if res {
            return c.get_result();
        }
    }

    return (
        justify_name("解析失败", 11),
        justify_name("解析失败", 11),
        justify_name("未启动", 9),
        justify_name("未启动", 7),
    );
}

fn main() {
    let args = Args::parse();

    let args_name = args.name.clone();
    if let Some(name) = args_name {
        print!("{}", justify_name(&name, 16));
        return;
    }

    env_logger::init();
    openssl_probe::init_ssl_cert_env_vars();

    let (download, upload, ping, jitter) = run(args);

    println!("{download},{upload},{ping},{jitter}");
}
