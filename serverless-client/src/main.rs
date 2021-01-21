use lambda::{handler_fn, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use mock::hello_ec2;
// use std::net::SocketAddr;
// use std::time::{Duration};
// use serde_json::json;

type Error = Box<dyn std::error::Error + Sync + Send + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda::run(handler_fn(handler)).await?;
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
    async fn run(&self) -> Result<Results, Error> {
        let res = Results::default();
        // should call mock-client function here, run place holder for now
        match hello_ec2(&self.addr).await {
            Ok(true) => Ok(res),
            Ok(false) => return Err("Unable to say hello".into()),
            Err(_) => return Err("Unexpected error".into()),
        }
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
    let res = args.run().await.unwrap();
    println!("We got results: {:?}", res);
    Ok(serde_json::to_value(res).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock::echo_server;
    use serde_json::json;

    #[tokio::test]
    async fn handler_handles() {
        let addr = String::from("127.0.0.1:25000");

        tokio::spawn(async move {
            // run a echo server in the loop
            echo_server(&addr).await.unwrap()
        });

        let event = json!({
            "addr": "127.0.0.1:25000",
            "duration": 10,
            "number": 1,
            "length": 1024,
            "rw_ratio": 100,
        });
        let expected = json!({
            "iops": 0,
            "req_num": 0,
            "p10": 0.0,
            "p50": 0.0,
            "p95": 0.0,
            "p99": 0.0
        });
        tokio::spawn(async move {
            // test if we can get the results correctly
            let results = handler(event, Context::default()).await.unwrap();
            assert_eq!(results, expected);
        });
    }
}
