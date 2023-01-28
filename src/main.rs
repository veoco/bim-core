use getopts::Options;
use std::env;

mod requests;
mod speedtest;
mod utils;
use speedtest::SpeedTest;
use utils::justify_name;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} DOWNLOAD_URL UPLOAD_URL [options]", program);
    print!("{}", opts.usage(&brief));
}

fn run(
    download_url: String,
    upload_url: String,
    ipv6: bool,
    connection_close: bool,
    multi_thread: bool,
) -> (String, String, String, String, String, String) {
    let client = SpeedTest::build(
        download_url,
        upload_url,
        ipv6,
        connection_close,
        multi_thread,
    );

    if let Some(mut c) = client {
        let _ = c.run();
        return c.get_result();
    }

    return (
        justify_name("解析失败", 9, false),
        justify_name("失败", 5, false),
        justify_name("解析失败", 9, false),
        justify_name("失败", 5, false),
        justify_name("未启动",7, false),
        justify_name("未启动", 7, false),
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("6", "ipv6", "enable ipv6");
    opts.optflag("c", "close", "enable connection close mode");
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
    let close = matches.opt_present("c");
    let multi = matches.opt_present("m");

    #[cfg(debug_assertions)]
    env_logger::init();

    let (upload, upload_status, download, download_status, ping, jitter) =
        run(download_url, upload_url, ipv6, close, multi);

    println!("{upload},{upload_status},{download},{download_status},{ping},{jitter}");
}
