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

use std::{env};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::TcpStream;
use tokio::net::{TcpSocket};
use tokio::sync::mpsc;

use std::net::SocketAddr;
use std::io;
use std::collections::HashMap;
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

#[derive(Debug)]
struct Count {
    send: u64,
    recv: u64,
    send_bytes: u64,
    recv_bytes: u64,
}


impl Default for Count {
    fn default () -> Count {
        Count {
            send: 0,
            recv: 0,
            send_bytes:0,
            recv_bytes: 0
        }
    }
}

#[derive(Debug)]
struct RTT {
    write: Duration,
    read: Duration,
    interval: Duration,
}

impl RTT {
    fn sort_write(&self) -> Duration {
        self.write
    }
    fn sort_read(&self) -> Duration {
        self.read
    }
    fn sort_interval(&self) -> Duration {
        self.write
    }
}

impl Default for RTT {
    fn default () -> RTT {
        RTT {
            write: Duration::default(),
            read: Duration::default(),
            interval: Duration::default(),
        }
    }
}

// Return RTT with all fields of specified percentile
fn percentile(n: usize, latency: &mut Vec<RTT>) -> RTT {
    if n > 100 {
        println!("Cannot calculate {}-percentile", n);
        RTT::default();
    }
    let mut res = RTT::default();
    let s = latency.len();
    let ind = s * n / 100;
    // A more efficient way needed, may be decouple all latency
    // fields in different data structures
    latency.sort_by_key(|k| k.sort_write());
    res.write = latency[ind].write;
    latency.sort_by_key(|k| k.sort_read());
    res.read = latency[ind].read;
    latency.sort_by_key(|k| k.sort_interval());
    res.interval = latency[ind].interval;

    res
}

//  Return RTT with all fields of average
fn average(latency: &mut Vec<RTT>) -> Duration {
    let s = latency.len();
    let mut sum = Duration::default();
    for rtt in latency {
        sum += rtt.write + rtt.read + rtt.interval;
    }
    Duration::from_secs_f64(sum.as_secs_f64() / s as f64)
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
        return Ok(())
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
    let (ltx, mut lrx) = mpsc::channel(4096);

    println!("Benchmarking:");
    println!(
        "{} streams will send {}-byte packets to {} for {} sec.",
        number, length, address, duration
    );

    for i in 0..number {
        // created for each tasks
        let tx = tx.clone();
        let ltx = ltx.clone();
        let address = address.clone();
        let length = length;

        tokio::spawn(async move {
            let mut sum = Count::default();
            let out_buf: Vec<u8> = vec![0; 4096];
            let mut in_buf: Vec<u8> = vec![0; 4096];
            let mut latency: Vec<RTT> = Vec::new();

            // Open a TCP stream to the socket address.
            let socket = TcpSocket::new_v4().unwrap();
            let mut stream = socket.connect(address).await.unwrap();

            let start = Instant::now();
            loop {
                let mut rtt = RTT::default();
                let mut last_t = Instant::now();
                match stream.write_all(&out_buf[0..length]).await {
                    Ok(_) => {
                        sum.send += 1;
                        sum.send_bytes += length as u64;
                        rtt.write = last_t.elapsed();
                        last_t = Instant::now();
                    }
                    Err(_) => {
                        println!("Write error!");
                        break;
                    }
                };

                rtt.interval = last_t.elapsed();
                last_t = Instant::now();
                match stream.read(&mut in_buf).await {
                    Ok(n) => {
                        sum.recv += 1;
                        sum.recv_bytes += n as u64;
                        rtt.read = last_t.elapsed();
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(_) => {
                        println!("Read error!");
                        break;
                    }
                };
                latency.push(rtt);
                let elapsed = start.elapsed();
                if elapsed > totltime {
                    // println!("Done benchamarking for task-{}", i);
                    break;
                }
            }
            // send packet data back
            tx.send(sum).await.expect("Send network statistics failed.");
            match ltx.send(latency).await {
                Ok(_) => true,
                Err(_) => {
                    println!("Send latency metrics from {}-task failed", i);
                    false
                }         
            }
            // Ok(())
        });
    }


    let mut sum = Count::default();
    let mut metrics = HashMap::new();
    for i in 0..number {
        let c = match rx.recv().await {
            Some(c) => c,
            None => Count::default(),
        };
        sum.recv += c.recv;
        sum.recv_bytes += c.recv_bytes;
        sum.send += c.send;
        sum.send_bytes += c.send_bytes;

        let latency = lrx.recv().await.unwrap();
        metrics.insert(i, latency);
    }
    
    let qps = sum.send as f64 / duration as f64 / 1000.0;
    let rps = sum.recv as f64 / duration as f64 / 1000.0;
    if qps == rps {
        println!();
        println!("The actual QPS is {:.1}K", rps);
    } else {
        println!("Requests {:.1} / Responses {:.1} mismatch", qps, rps);
        return Ok(())
    }

    println!();

    let pts = vec![10, 50, 95, 99];

    let mut ic = 0;
    for (i, mut latency) in metrics {
        if ic >= 10 {
            break;
        } else {
            ic += 1;
        }

        println!("Stream-{}:", i);
        for pt in &pts {
            let res = percentile(*pt, &mut latency);
            print!("| {}th-{:?} |", pt, res);
        }
        println!();
        let avg = average(&mut latency);
        println!("Average: {:?}", avg);
        println!();
    }
    println!();
    println!(
        "Send: {} bytes / {} Mbps",
        sum.send_bytes,
        sum.send_bytes / duration / 1024 / 128
    );
    println!(
        "Recv: {} bytes / {} Mbps",
        sum.recv_bytes,
        sum.recv_bytes / duration / 1024 / 128
    );
    Ok(())
}