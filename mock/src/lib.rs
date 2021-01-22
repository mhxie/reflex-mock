#![warn(rust_2018_idioms)]

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use std::error::Error;

pub async fn echo_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0; 4096];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                let resp = match n {
                    0 => return,
                    24 => 1048,
                    1048 => 24,
                    _ => n,
                };

                socket
                    .write_all(&buf[0..resp])
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
}

pub async fn hello_ec2(addr: &str) -> Result<bool, Box<dyn Error>> {
    // simple hello world function to test VPC connectivity
    println!("connecting server @{}", addr);
    let mut stream = TcpStream::connect(&addr).await?;
    // let mut stream = TcpStream::connect_timeout(addr, Duration::from_secs(1)).await?;
    println!("created stream");

    let result = stream.write(b"hello world\n").await;
    println!("wrote to stream; success={:?}", result.is_ok());
    Ok(result.is_ok())
}

pub async fn pressure_ec2(
    addr: &str,
    _duration: u64,
    _conns: u32,
    _length: usize,
    _rw_ratio: u32,
) -> Result<(), Box<dyn Error>> {
    // pressure a single server to get the peak performance
    println!("pressuring server @{}", addr);
    Ok(())
}

pub async fn pressure_multi_ec2(
    addrs: &[String],
    _duration: u64,
    _conns: u32,
    _length: usize,
    _rw_ratio: u32,
) -> Result<(), Box<dyn Error>> {
    // split data stream to multiple servers
    for addr in addrs {
        println!("pressuring servers @{}", &addr);
    }
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
