use getopts::Options;
use std::env;

use bim_core::clients::{Client, HTTPClient, SpeedtestNetTcpClient};
use bim_core::utils::{justify_name, SpeedTestResult};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} DOWNLOAD_URL UPLOAD_URL [options]", program);
    print!("{}", opts.usage(&brief));
}

fn get_client(
    client_name: &str,
    download_url: String,
    upload_url: String,
    ipv6: bool,
    threads: u8,
) -> Option<Box<dyn Client>> {
    match client_name {
        "http" => HTTPClient::build(download_url, upload_url, ipv6, threads),
        "tcp" => SpeedtestNetTcpClient::build(upload_url, ipv6, threads),
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "client", "set test client", "NAME");
    opts.optflagopt("m", "multi", "enable multi threads", "NUM");
    opts.optflag("6", "ipv6", "enable ipv6");
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

    let theads = {
        if matches.opt_present("m") {
            if let Some(value) = matches.opt_str("m") {
                match value.parse() {
                    Ok(m) => m,
                    Err(_) => {
                        print_usage(&program, opts);
                        return;
                    }
                }
            } else {
                8
            }
        } else {
            1
        }
    };

    #[cfg(debug_assertions)]
    env_logger::init();

    let client_name = matches.opt_str("c").unwrap_or("http".to_string());
    let r = {
        if let Some(mut client) = get_client(&client_name, download_url, upload_url, ipv6, theads) {
            let _ = (*client).run();
            client.result()
        } else {
            SpeedTestResult::build(0.0, "失败".to_string(), 0.0, "失败".to_string(), 0.0, 0.0)
        }
    };

    println!("{}", r.text());
}
