extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Fields};

/// Attribute macro for OVSDB table structs
///
/// This macro automatically adds `_uuid` and `_version` fields to your struct
/// and generates the necessary implementations for it to work with OVSDB.
///
/// # Example
///
/// ```rust
/// use ovsdb_derive::ovsdb_object;
/// use std::collections::HashMap;
///
/// #[ovsdb_object]
/// pub struct NbGlobal {
///     pub name: Option<String>,
///     pub nb_cfg: Option<i64>,
///     pub external_ids: Option<HashMap<String, String>>,
/// }
/// ```
#[proc_macro_attribute]
pub fn ovsdb_object(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the struct definition
    let mut input = parse_macro_input!(item as DeriveInput);

    // Add _uuid and _version fields if they don't exist
    if let Data::Struct(ref mut data_struct) = input.data {
        if let Fields::Named(ref mut fields) = data_struct.fields {
            // Check if _uuid and _version already exist
            let has_uuid = fields
                .named
                .iter()
                .any(|f| f.ident.as_ref().is_some_and(|i| i == "_uuid"));
            let has_version = fields
                .named
                .iter()
                .any(|f| f.ident.as_ref().is_some_and(|i| i == "_version"));

            // Add fields if they don't exist
            if !has_uuid {
                // Add _uuid field
                fields.named.push(parse_quote! {
                    pub _uuid: Option<uuid::Uuid>
                });
            }
            if !has_version {
                // Add _version field
                fields.named.push(parse_quote! {
                    pub _version: Option<uuid::Uuid>
                });
            }
        }
    }

    // Get the name of the struct
    let struct_name = &input.ident;

    // Extract field names and types, excluding _uuid and _version
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();

    if let Data::Struct(ref data_struct) = input.data {
        if let Fields::Named(ref fields) = data_struct.fields {
            for field in &fields.named {
                if let Some(ident) = &field.ident {
                    if ident == "_uuid" || ident == "_version" {
                        continue;
                    }
                    field_names.push(ident);
                    field_types.push(&field.ty);
                }
            }
        }
    }

    // Generate implementations
    let implementation = quote! {
        // Re-export the input struct with the added fields
        #input

        // Automatically import necessary items from ovsdb-schema
        use ::ovsdb_schema::{extract_uuid, OvsdbSerializableExt};

        impl #struct_name {
            /// Create a new instance with default values
            pub fn new() -> Self {
                Self {
                    #(
                        #field_names: Default::default(),
                    )*
                    _uuid: None,
                    _version: None,
                }
            }

            /// Convert to a HashMap for OVSDB serialization
            pub fn to_map(&self) -> std::collections::HashMap<String, serde_json::Value> {
                let mut map = std::collections::HashMap::new();

                #(
                    // Skip None values
                    let field_value = &self.#field_names;
                    if let Some(value) = field_value.to_ovsdb_json() {
                        map.insert(stringify!(#field_names).to_string(), value);
                    }
                )*

                map
            }

            /// Create from a HashMap received from OVSDB
            pub fn from_map(map: &std::collections::HashMap<String, serde_json::Value>) -> Result<Self, String> {
                let mut result = Self::new();

                // Extract UUID if present
                if let Some(uuid_val) = map.get("_uuid") {
                    if let Some(uuid) = extract_uuid(uuid_val) {
                        result._uuid = Some(uuid);
                    }
                }

                // Extract version if present
                if let Some(version_val) = map.get("_version") {
                    if let Some(version) = extract_uuid(version_val) {
                        result._version = Some(version);
                    }
                }

                // Extract other fields
                #(
                    if let Some(value) = map.get(stringify!(#field_names)) {
                        result.#field_names = <#field_types>::from_ovsdb_json(value)
                            .ok_or_else(|| format!("Failed to parse field {}", stringify!(#field_names)))?;
                    }
                )*

                Ok(result)
            }
        }

        impl Default for #struct_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl serde::Serialize for #struct_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer
            {
                self.to_map().serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for #struct_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>
            {
                let map = std::collections::HashMap::<String, serde_json::Value>::deserialize(deserializer)?;
                Self::from_map(&map).map_err(serde::de::Error::custom)
            }
        }
    };

    // Return the modified struct and implementations
    TokenStream::from(implementation)
}

/// Derive macro for OVSDB table structs (requires manual _uuid and _version fields)
///
/// This macro generates the necessary implementations for a struct to work with OVSDB.
/// The struct must have `_uuid` and `_version` fields of type `Option<uuid::Uuid>`.
///
/// # Example
///
/// ```rust
/// use ovsdb_derive::OVSDB;
/// use std::collections::HashMap;
/// use uuid::Uuid;
///
/// #[derive(Debug, Clone, PartialEq, OVSDB)]
/// pub struct NbGlobal {
///     pub name: Option<String>,
///     pub nb_cfg: Option<i64>,
///     pub external_ids: Option<HashMap<String, String>>,
///     
///     // Required fields
///     pub _uuid: Option<Uuid>,
///     pub _version: Option<Uuid>,
/// }
/// ```
#[proc_macro_derive(OVSDB)]
pub fn ovsdb_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
    let struct_name = &input.ident;

    // Check if the input is a struct
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => panic!("OVSDB can only be derived for structs with named fields"),
        },
        _ => panic!("OVSDB can only be derived for structs"),
    };

    // Extract field names and types, excluding _uuid and _version
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();

    for field in fields {
        if let Some(ident) = &field.ident {
            if ident == "_uuid" || ident == "_version" {
                continue;
            }
            field_names.push(ident);
            field_types.push(&field.ty);
        }
    }

    // Generate code for the implementation
    let expanded = quote! {
        // Automatically import necessary items from ovsdb-schema
        use ::ovsdb_schema::{extract_uuid, OvsdbSerializableExt};

        impl #struct_name {
            /// Create a new instance with default values
            pub fn new() -> Self {
                Self {
                    #(
                        #field_names: Default::default(),
                    )*
                    _uuid: None,
                    _version: None,
                }
            }

            /// Convert to a HashMap for OVSDB serialization
            pub fn to_map(&self) -> std::collections::HashMap<String, serde_json::Value> {
                let mut map = std::collections::HashMap::new();

                #(
                    // Skip None values
                    let field_value = &self.#field_names;
                    if let Some(value) = field_value.to_ovsdb_json() {
                        map.insert(stringify!(#field_names).to_string(), value);
                    }
                )*

                map
            }

            /// Create from a HashMap received from OVSDB
            pub fn from_map(map: &std::collections::HashMap<String, serde_json::Value>) -> Result<Self, String> {
                let mut result = Self::new();

                // Extract UUID if present
                if let Some(uuid_val) = map.get("_uuid") {
                    if let Some(uuid) = extract_uuid(uuid_val) {
                        result._uuid = Some(uuid);
                    }
                }

                // Extract version if present
                if let Some(version_val) = map.get("_version") {
                    if let Some(version) = extract_uuid(version_val) {
                        result._version = Some(version);
                    }
                }

                // Extract other fields
                #(
                    if let Some(value) = map.get(stringify!(#field_names)) {
                        result.#field_names = <#field_types>::from_ovsdb_json(value)
                            .ok_or_else(|| format!("Failed to parse field {}", stringify!(#field_names)))?;
                    }
                )*

                Ok(result)
            }
        }

        impl Default for #struct_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl serde::Serialize for #struct_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer
            {
                self.to_map().serialize(serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for #struct_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>
            {
                let map = std::collections::HashMap::<String, serde_json::Value>::deserialize(deserializer)?;
                Self::from_map(&map).map_err(serde::de::Error::custom)
            }
        }
    };

    // Return the generated code
    TokenStream::from(expanded)
}
