use getopts::Options;
use std::env;

use bim_core::servers::{HTTPServer, Server};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} HOST:PORT [options]", program);
    print!("{}", opts.usage(&brief));
}

fn get_server(server_name: &str, address: &str) -> Option<Box<dyn Server>> {
    match server_name {
        "http" => Some(Box::new(HTTPServer::build(address.to_string()).unwrap())),
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("s", "server", "set test server", "NAME");
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

    let address = if !matches.free.is_empty() {
        matches.free.get(0).unwrap()
    } else {
        print_usage(&program, opts);
        return;
    };

    #[cfg(debug_assertions)]
    env_logger::init();

    let server_name = matches.opt_str("s").unwrap_or("http".to_string());
    if let Some(mut server) = get_server(&server_name, address) {
        println!("Running {server_name} server on: {address}");
        let _ = (*server).run();
    } else {
        println!("{server_name} client not found or invalid params.")
    }
}
