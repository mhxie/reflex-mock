//! Hello world server.
//!
//! A simple client that opens a TCP stream, writes "hello world\n", and closes
//! the connection.
//!
//! You can test this out by running:
//!
//!     ncat -l 25000
//!

#![warn(rust_2018_idioms)]

use std::env;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::TcpStream;
use tokio::net::{TcpSocket};
use tokio::sync::mpsc;

use std::net::SocketAddr;
use std::io;
use std::error::Error;
use std::time::{Duration, Instant};

fn print_usage(program: &str, opts: &getopts::Options) {
    let brief = format!(
        r#"Echo benchmark.

Usage:
  {program} [ -a <address> ] [ -l <length> ] [ -c <number> ] [ -t <duration> ]
  {program} (-h | --help)
  {program} --version"#,
        program = program
    );
    print!("{}", opts.usage(&brief));
}

// #[derive(Debug)]
struct Count {
    inb: u64,
    outb: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "Print this help.");
    opts.optopt(
        "a",
        "address",
        "Target echo server address. Default: 127.0.0.1:25000",
        "<address>",
    );
    opts.optopt(
        "l",
        "length",
        "Test message length. Default: 1024",
        "<length>",
    );
    opts.optopt(
        "t",
        "duration",
        "Test duration in seconds. Default: 10",
        "<duration>",
    );
    opts.optopt(
        "c",
        "number",
        "Test connection number. Default: 10",
        "<number>",
    );

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e.to_string());
            print_usage(&program, &opts);
            return Err(e.into());
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return Ok(());
    }

    let length = matches
        .opt_str("length")
        .unwrap_or_default()
        .parse::<usize>()
        .unwrap_or(1024);

    if length > 4096 {
        println!("Please specify packet size equal or smaller than 4096 bytes.");
        return Ok(());
    }

    let duration = matches
        .opt_str("duration")
        .unwrap_or_default()
        .parse::<u64>()
        .unwrap_or(10);
    let number = matches
        .opt_str("number")
        .unwrap_or_default()
        .parse::<u32>()
        .unwrap_or(10);
    let address = matches
        .opt_str("address")
        .unwrap_or_else(|| "127.0.0.1:25000".to_string())
        .parse::<SocketAddr>()
        .unwrap();

    let totltime = Duration::from_secs(duration);
    let (tx, mut rx) = mpsc::channel(32);

    for i in 0..number {
        // created for each tasks
        let tx = tx.clone();
        let address = address.clone();
        let length = length;

        tokio::spawn(async move {
            let mut sum = Count { inb: 0, outb: 0 };
            let out_buf: Vec<u8> = vec![0; 4096];
            let mut in_buf: Vec<u8> = vec![0; 4096];

            // Open a TCP stream to the socket address.
            let socket = TcpSocket::new_v4().unwrap();
            let mut stream = socket.connect(address).await.unwrap();
            // println!("created stream-{}", i);

            let start = Instant::now();
            loop {
                match stream.write_all(&out_buf[0..length]).await {
                    Err(_) => {
                        println!("Write error!");
                        break;
                    }
                    Ok(_) => sum.outb += 1,
                };

                match stream.read(&mut in_buf).await {
                    Ok(_) => sum.inb += 1,
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(_) => {
                        println!("Read error!");
                        break;
                    }
                };
                let elapsed = start.elapsed();
                if elapsed > totltime {
                    // println!("Done benchamarking for task-{}", i);
                    break;
                }
            }
            match tx.send(sum).await {
                Ok(_) => true,
                Err(_) => {
                    println!("Send sum from {}-task failed", i);
                    false
                }
            }
            // Ok(())
        });
    }

    let mut sum = Count { inb: 0, outb: 0 };
    for _ in 0..number {
        let c = match rx.recv().await {
            Some(c) => c,
            None => Count { inb: 0, outb: 0 },
        };
        sum.inb += c.inb;
        sum.outb += c.outb;
    }

    println!("Benchmarking: {}", address);
    println!(
        "{} streams, running {} bytes for {} sec.",
        number, length, duration
    );
    println!();
    println!("Requests: {} Responses: {}", sum.outb, sum.inb);
    println!(
        "Speed: {} requests/sec, {} Mbps, {} responses/sec",
        sum.outb / duration,
        sum.outb / duration * (length as u64) / 1024 / 128,
        sum.inb / duration
    );
    Ok(())
}