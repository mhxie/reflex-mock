use lambda::{handler_fn, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;
// use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
// use std::time::{Duration};
// use serde_json::json;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda::run(handler_fn(handler)).await?;
    Ok(())
}

async fn hello_ec2(addr: &str) -> Result<(), Error> {
    let mut stream = TcpStream::connect(addr).await?;
    // let mut stream = TcpStream::connect_timeout(addr, Duration::from_secs(1)).await?;
    println!("created stream");

    let result = stream.write(b"hello world\n").await;
    println!("wrote to stream; success={:?}", result.is_ok());
    Ok(())
}

#[derive(Deserialize, Debug)]
struct Args {
    addr: String,
    duration: u64,
    number: u32,
    length: usize,
    rw_ratio: u32,
}

impl Args {
    async fn run(&self) -> Results {
        let res = Results::default();
        // call mock-client function here
        hello_ec2(&self.addr).await.expect("Unable to say hello");
        res
    }
}

#[derive(Serialize, Debug)]
struct Results {
    iops: u64,
    req_num: u64,
    // tail latency, flat structure
    p10: f64,
    p50: f64,
    p95: f64,
    p99: f64,
}

impl Default for Results {
    fn default() -> Self {
        Results {
            iops: 0,
            req_num: 0,
            p10: 0.0,
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
        }
    }
}

async fn handler(event: serde_json::Value, _: Context) -> Result<Value, Error> {
    let args: Args = serde_json::from_value(event).unwrap();
    println!("We got args: {:?}", args);
    let res = args.run().await;
    println!("We got results: {:?}", res);
    Ok(serde_json::to_value(res).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn handler_handles() {
        let event = json!({
            "addr": "10.0.101.78:25000",
            "duration": 10,
            "number": 1,
            "length": 1024,
            "rw_ratio": 100,
        });
        let results = json!({
            "iops": 0,
            "req_num": 0,
            "p10": 0.0,
            "p50": 0.0,
            "p95": 0.0,
            "p99": 0.0
        });
        assert_eq!(
            handler(event, Context::default())
                .await
                .expect("expected json result value"),
            results
        )
    }
}
