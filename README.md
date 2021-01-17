# ReFlex-Mock

ReFlex-Mock is consist of server and client mock implementations for benchmarking various workloads.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/mhxie/reflex-mock/blob/main/LICENSE)
[![Build Status](https://github.com/mhxie/reflex-mock/workflows/CI/badge.svg)](https://github.com/mhxie/reflex-mock/actions?query=workflow%3ACI)
[![Deploy Status](https://github.com/mhxie/reflex-mock/workflows/CD/badge.svg)](https://github.com/mhxie/reflex-mock/actions?query=workflow%3ACD)

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

    # Try the serverless demo with the plugin serverless-rust
    docker pull softprops/lambda-rust:0.2.7-rust-1.43.1
    cd serverless-client && npm ci
    npx serverless deploy

    # Test your invocation and have fun
    npx serverless invoke -f hello -d '{"foo":"bar"}'

## References

* https://github.com/haraldh/rust_echo_bench
* https://github.com/tokio-rs/tokio
* https://www.serverless.com/plugins/serverless-rust
