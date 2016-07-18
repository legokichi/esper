#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate rustc_serialize;
extern crate docopt;
extern crate hyper;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate esper;

use std::thread;
use std::sync::{Arc, Mutex};

use hyper::net::{HttpListener};
use hyper::server::{Server};

use esper::{Config, Manager};
use esper::handler::EventStream;

docopt!(Args derive Debug, "
esper - Event Source HTTP server, powered by hyper.

Usage:
  esper [-n | --no-auth] [--bind=<bind>] [--port=<port>] [--threads=<st>]
  esper (-h | --help)
  esper (-v | --version)

Options:
  -h --help          Show this screen.
  -v --version       Show version.
  -b --bind=<bind>   Bind to specific IP [default: 127.0.0.1]
  -p --port=<port>   Run on a specific port number [default: 3000]
  -t --threads=<st>  Number of server threads [default: 2].
  --no-auth          Run without JWT authentication.
", flag_threads: u8);

fn main() {
    println!("Welcome to esper -- the Event Source HTTP server, powered by hyper!\n");
    env_logger::init().unwrap();

    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    debug!("Executing with args: {:?}", args);

    if args.flag_version {
        println!("esper v0.1.0");
        std::process::exit(0);
    }

    let addr = format!("{}:{}", args.flag_bind, args.flag_port);
    let listener = HttpListener::bind(&addr.parse().unwrap()).unwrap();

    let mut handles = Vec::new();

    let mgr: Manager = Manager::new();
    let mgr_ref = Arc::new(Mutex::new(mgr));

    let cfg_ref = Arc::new(Config::from_env());

    for _ in 0..args.flag_threads {
        let listener = listener.try_clone().unwrap();

        let cfg_inner = cfg_ref.clone();
        let mgr_inner = mgr_ref.clone();

        handles.push(thread::spawn(move || {
            Server::new(listener).handle(|ctrl| {
                EventStream::new(cfg_inner.clone(), ctrl, mgr_inner.clone())
            }).unwrap();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
