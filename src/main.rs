use std::env;
use std::io::{self, Write};
use std::net::{IpAddr, TcpStream};
use std::str::FromStr;
use std::process;
use std::sync::mpsc::{Sender, channel};
use std::thread;

const MAX_U16: u16 = 65535;

struct Arguments {
    flag: String,
    ipaddr: IpAddr,
    threads: u16,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        } else if args.len() > 4 {
            return Err("too many arguments");
        }

        let f = args[1].clone();
        if let Ok(ipaddr) = IpAddr::from_str(&f) {
            return Ok(Arguments { flag: String::new(), ipaddr, threads: 4 });
        } else {
            let flag = args[1].clone();
            if flag == "-h" || flag == "--help" {
                println!("Usage: -j <num_threads> <ip_address>");
                return Err("help");
            } else if flag == "-j" {
                if args.len() != 4 {
                    return Err("invalid number of arguments for -j");
                }

                let threads = match args[2].parse::<u16>() {
                    Ok(s) => s,
                    Err(_) => return Err("invalid thread number"),
                };

                let ipaddr = match IpAddr::from_str(&args[3]) {
                    Ok(s) => s,
                    Err(_) => return Err("not a valid IPADDR: must be IPv4 or IPv6"),
                };

                return Ok(Arguments { flag, ipaddr, threads });
            } else {
                return Err("invalid syntax");
            }
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port = start_port;

    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port).unwrap();
            }
            Err(_) => {}
        }

        if (MAX_U16 - port) <= num_threads {
            break;
        }
        port += num_threads;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let arguments = Arguments::new(&args).unwrap_or_else(|err| {
        if err == "help" {
            process::exit(0);
        } else {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    });

    let num_threads = arguments.threads;
    let (tx, rx) = channel();

    for i in 0..num_threads {
        let tx = tx.clone();
        let ipaddr = arguments.ipaddr;

        thread::spawn(move || {
            scan(tx, i, ipaddr, num_threads);
        });
    }

    let mut out = vec![];
    drop(tx);

    for p in rx {
        out.push(p);
    }

    println!();
    out.sort();
    for v in out {
        println!("{} is open", v);
    }
}
