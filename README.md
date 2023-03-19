# Beacon API Checker

CLI tool for cross-checking beacon API responses across clients

## Installing

Dependencies:

- Rust 1.65.0 stable toolchain

```bash
cargo install
```

## Running

```
Usage: api-checker [OPTIONS] [METRICS_HOST] [METRICS_PORT] [INTERVAL_SECONDS] [HTTP_TIMEOUT]

Arguments:
  [METRICS_HOST]      [default: 127.0.0.1]
  [METRICS_PORT]      [default: 8080]
  [INTERVAL_SECONDS]  
  [HTTP_TIMEOUT]      

Options:
      --beacon-api-endpoints <BEACON_API_ENDPOINTS>  
  -h, --help                                         Print help
  -V, --version                                      Print version
```

## Example

```
cargo run -- --beacon-api-endpoints="http://localhost:3501" --beacon-api-endpoints="http://localhost:3500"

   Compiling api-checker v0.1.0 (/home/code/rust/api-checker)
    Finished dev [unoptimized + debuginfo] target(s) in 4.87s
     Running `target/debug/api-checker
2023-03-19T01:58:19.414517Z  INFO api_checker: Starting API checker
2023-03-19T01:58:19.414611Z  INFO api_checker: Starting prometheus metrics server
2023-03-19T01:58:19.415726Z  INFO api_checker: Running API checker pipeline
2023-03-19T01:58:24.850151Z  INFO api_checker::endpoints: Got equal /eth/v1/beacon/validators 2023-03-19T01:58:25.191757Z  INFO api_checker::endpoints: Got equal /eth/v1/beacon/balances re2023-03-19T01:59:19.415605Z  INFO api_checker: Running API checker pipeline
2023-03-19T01:59:19.772738Z  INFO api_checker::endpoints: Got equal /eth/v1/beacon/validators 2023-03-19T01:59:20.094600Z  INFO api_checker::endpoints: Got equal /eth/v1/beacon/balances re
```

