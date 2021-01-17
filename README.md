# ReFlex-Mock

ReFlex-Mock is consist of server and client mock implementations for benchmarking various workloads.

## Build

    cargo build

## Run Mock Server

    cargo run --release -p mock-server

## Run Mock Client

    cargo run --release -p mock-client
    cargo run --release -p mock-client -- --help
    cargo run --release -p mock-client -- --address "127.0.0.1:25000" --number 1000 --duration 60 --length 1024

## Go Serverless

    # Configure serverless and AWS-cli
    curl -o- -L https://slss.io/install | bash
    pip3 install awscli --upgrade --user
    aws configure

    # Try the serverlessdemo with the plugin serverless-rust
    docker pull softprops/lambda-rust:0.2.7-rust-1.43.1
    cd serverless-echo && npm ci
    npx serverless deploy

    # Test your invocation and have fun
    npx serverless invoke -f hello -d '{"foo":"bar"}'

## References

* https://github.com/haraldh/rust_echo_bench
* https://github.com/tokio-rs/tokio
* https://www.serverless.com/plugins/serverless-rust
