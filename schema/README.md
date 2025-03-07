# ovsdb-schema

A Rust implementation of the OVSDB protocol serialization and deserialization types.

## Overview

This crate provides the core primitives and traits needed to work with the Open vSwitch Database Management Protocol (OVSDB) as defined in [RFC7047](https://datatracker.ietf.org/doc/html/rfc7047). It includes:

- Type definitions for OVSDB data structures
- Serialization/deserialization between Rust types and OVSDB JSON format
- Traits to make your own types compatible with OVSDB

This crate is designed to be used alongside `ovsdb-derive` for a complete OVSDB client implementation.

## Features

- `OvsdbAtom` and `OvsdbValue` types representing OVSDB's basic data types
- `OvsdbSerializable` trait for converting between Rust types and OVSDB values
- Implementations for common Rust types like `String`, `i64`, `bool`, etc.
- Support for collections like `Vec<T>` and `HashMap<K, V>`
- Helper functions for UUID handling
- Full support for OVSDB's type system: atoms, sets, and maps

## Usage

### Basic Usage

```rust
use ovsdb_schema::{OvsdbSerializable, OvsdbSerializableExt};
use std::collections::HashMap;
use uuid::Uuid;

// Use the trait directly
let my_string = "hello".to_string();
let ovsdb_value = my_string.to_ovsdb();
let json_value = my_string.to_ovsdb_json().unwrap();

// Extract UUIDs from JSON values
let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
let uuid = Uuid::parse_str(uuid_str).unwrap();
let json_value = serde_json::json!(["uuid", uuid_str]);
let extracted_uuid = ovsdb_schema::extract_uuid(&json_value).unwrap();
assert_eq!(uuid, extracted_uuid);
```

### With `ovsdb-derive`

This crate is designed to work with the companion `ovsdb-derive` crate:

```rust
use ovsdb_derive::ovsdb_object;
use std::collections::HashMap;

#[ovsdb_object]
pub struct NbGlobal {
    pub name: Option<String>,
    pub nb_cfg: Option<i64>,
    pub external_ids: Option<HashMap<String, String>>,
}

// The macro adds _uuid and _version fields and implements
// OvsdbSerializable automatically
```

## Type Conversion

| Rust Type | OVSDB Type |
|-----------|------------|
| `String` | string |
| `i64` | integer |
| `f64` | real |
| `bool` | boolean |
| `Uuid` | uuid |
| `Vec<T>` | set |
| `HashMap<K, V>` | map |
| `Option<T>` | value or empty set |

## Custom Types

Implement `OvsdbSerializable` for your custom types:

```rust
use ovsdb_schema::{OvsdbSerializable, OvsdbValue, OvsdbAtom};

struct MyType(String);

impl OvsdbSerializable for MyType {
    fn to_ovsdb(&self) -> OvsdbValue {
        OvsdbValue::Atom(OvsdbAtom::String(self.0.clone()))
    }

    fn from_ovsdb(value: &OvsdbValue) -> Option<Self> {
        match value {
            OvsdbValue::Atom(OvsdbAtom::String(s)) => Some(MyType(s.clone())),
            _ => None,
        }
    }
}
```

## License

This project is licensed under the [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0).
