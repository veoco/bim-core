use getopts::Options;
use std::env;

use bimc::clients::{Client, HTTPClient};
use bimc::utils::{justify_name, SpeedTestResult};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} DOWNLOAD_URL UPLOAD_URL [options]", program);
    print!("{}", opts.usage(&brief));
}

fn run(
    download_url: String,
    upload_url: String,
    ipv6: bool,
    multi_thread: bool,
) -> SpeedTestResult {
    let client = HTTPClient::build(download_url, upload_url, ipv6, multi_thread);

    if let Some(mut c) = client {
        let _ = c.run();
        return c.result();
    }

    SpeedTestResult::build(0.0, "失败".to_string(), 0.0, "失败".to_string(), 0.0, 0.0)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("6", "ipv6", "enable ipv6");
    opts.optflag("m", "multi", "enable multi thread");
    opts.optflag("n", "name", "print justified name");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}\n", f.to_string());
            print_usage(&program, opts);
            return;
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let (dl, ul) = if !matches.free.is_empty() {
        (matches.free.get(0), matches.free.get(1))
    } else {
        print_usage(&program, opts);
        return;
    };

    if matches.opt_present("n") {
        if let Some(name) = dl {
            print!("{}", justify_name(name, 12, true));
        } else {
            print_usage(&program, opts);
        }
        return;
    }

    if ul.is_none() {
        print_usage(&program, opts);
        return;
    }

    let download_url = dl.unwrap().clone();
    let upload_url = ul.unwrap().clone();
    let ipv6 = matches.opt_present("6");
    let multi = matches.opt_present("m");

    #[cfg(debug_assertions)]
    env_logger::init();

    let r = run(download_url, upload_url, ipv6, multi);

    println!("{}", r.text());
}
