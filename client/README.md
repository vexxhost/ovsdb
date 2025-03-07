# ovsdb-client

A Rust implementation of the OVSDB protocol client based on [RFC7047](https://datatracker.ietf.org/doc/html/rfc7047).

## Overview

This crate provides a client implementation for the Open vSwitch Database Management Protocol (OVSDB), allowing Rust applications to:

- Connect to OVSDB servers over TCP or Unix sockets
- Query database schemas
- Monitor tables for changes in real-time
- Execute transactions against OVSDB databases

## Features

- **Multiple Transport Options**: Connect via TCP or Unix socket
- **Schema Handling**: Retrieve and parse database schemas
- **Monitoring**: Subscribe to changes in database tables
- **JSON-RPC**: Built on top of `jsonrpsee` for reliable RPC communication
- **Async API**: Fully async API designed for use with Tokio

## Quick Start

```rust
use ovsdb_client::{
    rpc::{self, RpcClient},
    schema::{MonitorRequest, UpdateNotification},
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to an OVSDB server on localhost
    let client = rpc::connect_tcp("127.0.0.1:6641").await?;

    // List available databases
    let databases = client.list_databases().await?;
    println!("Available databases: {:?}", databases);

    // Get schema for a specific database
    let schema = client.get_schema("OVN_Northbound").await?;

    // Set up monitoring for a table
    let mut requests = HashMap::new();
    requests.insert(
        "NB_Global".to_owned(),
        MonitorRequest {
            columns: Some(vec!["name".to_owned(), "nb_cfg".to_owned()]),
            ..Default::default()
        },
    );

    // Start monitoring and get initial state
    let initial = client.monitor("OVN_Northbound", None, requests).await?;
    println!("Initial state: {:?}", initial);

    // Subscribe to updates
    let mut stream = client.subscribe_to_method("update").await?;
    while let Some(update) = stream.next().await {
        match update {
            Ok(update) => println!("Received update: {:?}", update),
            Err(e) => eprintln!("Error: {:?}", e),
        }
    }

    Ok(())
}
```

## API Overview

### Connections

```rust
// Connect via TCP
let client = rpc::connect_tcp("127.0.0.1:6641").await?;

// Connect via Unix socket
let client = rpc::connect_unix("/var/run/openvswitch/db.sock").await?;
```

### Basic Operations

```rust
// List databases
let databases = client.list_databases().await?;

// Get schema
let schema = client.get_schema("OVN_Northbound").await?;
```

### Monitoring

```rust
// Create monitor request
let mut requests = HashMap::new();
requests.insert(
    "Table_Name".to_owned(),
    MonitorRequest {
        columns: Some(vec!["column1".to_owned(), "column2".to_owned()]),
        ..Default::default()
    },
);

// Start monitoring
let initial_state = client.monitor("Database_Name", None, requests).await?;

// Subscribe to updates
let mut stream = client.subscribe_to_method("update").await?;
while let Some(update) = stream.next().await {
    // Process updates
}
```

## Development Setup

To develop or test with this crate, you'll need an OVSDB server. You can use Docker to run one:

```bash
docker run -it --rm -p 6641:6641 registry.atmosphere.dev/library/ovn-central:main /bin/bash -c "mkdir /etc/ovn; /root/ovnkube.sh nb-ovsdb"
```

This starts an OVN Northbound OVSDB server that listens on port 6641.

## OVSDB Protocol Support

This implementation supports the following OVSDB operations as defined in RFC7047:

- List Databases (Section 4.1.1)
- Get Schema (Section 4.1.2)
- Monitor (Section 4.1.5)
- Update Notifications (Section 4.1.6)

Future versions will add support for additional operations such as Transact (Section 4.1.3) and Monitor Cancellation (Section 4.1.7).

## Related Crates

- [ovsdb-schema](https://crates.io/crates/ovsdb-schema): Core OVSDB data types and serialization
- [ovsdb-derive](https://crates.io/crates/ovsdb-derive): Derive macros for OVSDB struct generation

## License

This project is licensed under the [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0).
