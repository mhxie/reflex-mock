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


## References

* https://github.com/haraldh/rust_echo_bench
* https://github.com/tokio-rs/tokio
