# OVSDB Rust

A collection of Rust crates for working with the Open vSwitch Database Management Protocol (OVSDB).

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

## Overview

This repository provides a complete Rust implementation of the OVSDB protocol as defined in [RFC7047](https://datatracker.ietf.org/doc/html/rfc7047). It's structured as a monorepo containing the following crates:

| Crate | Description | Status |
|-------|-------------|--------|
| [`ovsdb-schema`](./schema) | Rust types and serialization for OVSDB | [![crates.io](https://img.shields.io/crates/v/ovsdb-schema.svg)](https://crates.io/crates/ovsdb-schema) |
| [`ovsdb-derive`](./derive) | Procedural macros for OVSDB struct generation | [![crates.io](https://img.shields.io/crates/v/ovsdb-derive.svg)](https://crates.io/crates/ovsdb-derive) |
| [`ovsdb-client`](./client) | Async client for the OVSDB protocol | [![crates.io](https://img.shields.io/crates/v/ovsdb-client.svg)](https://crates.io/crates/ovsdb-client) |

## Features

- **Complete Type System**: Full implementation of OVSDB's type system (atoms, sets, maps)
- **Auto-generated Structs**: Derive macros for creating OVSDB-compatible structs
- **Async Client**: Modern async client using Tokio and jsonrpsee
- **Multiple Transports**: Support for TCP and Unix socket connections
- **Table Monitoring**: Real-time monitoring of table changes

## Quick Example

```rust
use ovsdb_derive::ovsdb_object;
use ovsdb_client::{rpc, schema::MonitorRequest};
use std::collections::HashMap;

#[ovsdb_object]
struct NbGlobal {
    name: Option<String>,
    nb_cfg: Option<i64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to an OVSDB server
    let client = rpc::connect_tcp("127.0.0.1:6641").await?;

    // Set up monitoring for the NB_Global table
    let mut requests = HashMap::new();
    requests.insert(
        "NB_Global".to_owned(),
        MonitorRequest {
            columns: Some(vec!["name".to_owned(), "nb_cfg".to_owned()]),
            ..Default::default()
        },
    );

    // Start monitoring
    let initial = client.monitor("OVN_Northbound", None, requests).await?;
    println!("Initial state: {:?}", initial);

    // Subscribe to updates
    let mut stream = client.subscribe_to_method("update").await?;
    while let Some(update) = stream.next().await {
        if let Ok(update) = update {
            println!("Received update: {:?}", update);
        }
    }

    Ok(())
}
```

## Getting Started

To use these crates in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
ovsdb-schema = "0.1.0"
ovsdb-derive = "0.1.0"
ovsdb-client = "0.1.0"
```

See the individual crate directories for more detailed documentation:
- [ovsdb-schema](./schema/README.md)
- [ovsdb-derive](./derive/README.md)
- [ovsdb-client](./client/README.md)

## Development

### Prerequisites

- Rust 1.75 or newer
- OVSDB server for testing (see below)

### Setting up a test environment

For development and testing, you can run an OVSDB server using Docker:

```bash
docker run -it --rm -p 6641:6641 registry.atmosphere.dev/library/ovn-central:main \
  /bin/bash -c "mkdir /etc/ovn; /root/ovnkube.sh nb-ovsdb"
```

### Running tests

```bash
cargo test --all
```

## License

This project is licensed under the [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0).
