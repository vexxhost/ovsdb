# OVSDB Derive

A procedural macro crate for Rust to generate code for OVSDB table structs.

## Overview

This crate provides two approaches for working with OVSDB tables:

- `#[ovsdb_object]` attribute macro: Automatically adds `_uuid` and `_version` fields to your struct
- `#[derive(OVSDB)]` derive macro: requires manual fields but offers more control

## Usage

You can either use the attribute macro or the derive macro to generate code for your OVSDB table structs. For more details
on how to use the library, check out the examples in the `examples` directory.

### Attribute Macro (Recommended)

```rust
use ovsdb_derive::ovsdb_object;
use std::collections::HashMap;

#[ovsdb_object]
pub struct NbGlobal {
    pub name: Option<String>,
    pub nb_cfg: Option<i64>,
    pub external_ids: Option<HashMap<String, String>>,
    // No need to add _uuid and _version fields
}
```

### Derive Macro (Alternative)

```rust
use ovsdb_derive::OVSDB;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, OVSDB)]
pub struct NbGlobal {
    pub name: Option<String>,
    pub nb_cfg: Option<i64>,
    pub external_ids: Option<HashMap<String, String>>,

    // Required fields with the derive approach
    pub _uuid: Option<Uuid>,
    pub _version: Option<Uuid>,
}
```

## Generated Code

Both macros generate the following implementations:

- `new()` method that creates a new instance with default values
- `to_map()` method that converts the struct to a HashMap for OVSDB serialization
- `from_map()` method that creates a struct from a HashMap received from OVSDB
- `Default` trait implementation
- `serde::Serialize` trait implementation
- `serde::Deserialize` trait implementation

## License

This project is licensed under the [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0).
