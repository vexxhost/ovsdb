use serde::{Serialize, Serializer};
use std::collections::HashMap;
use uuid::Uuid;

/// Primitive OVSDB Atom types
#[derive(Debug, Clone, PartialEq)]
pub enum OvsdbAtom {
    String(String),
    Integer(i64),
    Real(f64),
    Boolean(bool),
    Uuid(Uuid),
    NamedUuid(String),
}

/// OVSDB Value types (atom, set, or map)
#[derive(Debug, Clone, PartialEq)]
pub enum OvsdbValue {
    Atom(OvsdbAtom),
    Set(Vec<OvsdbAtom>),
    Map(Vec<(OvsdbAtom, OvsdbAtom)>),
}

/// Trait for converting between Rust types and OVSDB Values
pub trait OvsdbSerializable: Sized {
    fn to_ovsdb(&self) -> OvsdbValue;
    fn from_ovsdb(value: &OvsdbValue) -> Option<Self>;
}

impl<T: OvsdbSerializable> OvsdbSerializable for Option<T> {
    fn to_ovsdb(&self) -> OvsdbValue {
        match self {
            Some(val) => val.to_ovsdb(),
            None => OvsdbValue::Set(vec![]), // Empty set for None
        }
    }

    fn from_ovsdb(value: &OvsdbValue) -> Option<Self> {
        T::from_ovsdb(value).map(Some)
    }
}

impl OvsdbSerializable for String {
    fn to_ovsdb(&self) -> OvsdbValue {
        OvsdbValue::Atom(OvsdbAtom::String(self.clone()))
    }

    fn from_ovsdb(value: &OvsdbValue) -> Option<Self> {
        match value {
            OvsdbValue::Atom(OvsdbAtom::String(s)) => Some(s.clone()),
            _ => None,
        }
    }
}

impl OvsdbSerializable for i64 {
    fn to_ovsdb(&self) -> OvsdbValue {
        OvsdbValue::Atom(OvsdbAtom::Integer(*self))
    }

    fn from_ovsdb(value: &OvsdbValue) -> Option<Self> {
        match value {
            OvsdbValue::Atom(OvsdbAtom::Integer(i)) => Some(*i),
            _ => None,
        }
    }
}

impl OvsdbSerializable for f64 {
    fn to_ovsdb(&self) -> OvsdbValue {
        OvsdbValue::Atom(OvsdbAtom::Real(*self))
    }

    fn from_ovsdb(value: &OvsdbValue) -> Option<Self> {
        match value {
            OvsdbValue::Atom(OvsdbAtom::Real(r)) => Some(*r),
            _ => None,
        }
    }
}

impl OvsdbSerializable for bool {
    fn to_ovsdb(&self) -> OvsdbValue {
        OvsdbValue::Atom(OvsdbAtom::Boolean(*self))
    }

    fn from_ovsdb(value: &OvsdbValue) -> Option<Self> {
        match value {
            OvsdbValue::Atom(OvsdbAtom::Boolean(b)) => Some(*b),
            _ => None,
        }
    }
}

impl OvsdbSerializable for Uuid {
    fn to_ovsdb(&self) -> OvsdbValue {
        OvsdbValue::Atom(OvsdbAtom::Uuid(*self))
    }

    fn from_ovsdb(value: &OvsdbValue) -> Option<Self> {
        match value {
            OvsdbValue::Atom(OvsdbAtom::Uuid(uuid)) => Some(*uuid),
            _ => None,
        }
    }
}

impl<T: OvsdbSerializable> OvsdbSerializable for Vec<T> {
    fn to_ovsdb(&self) -> OvsdbValue {
        if self.is_empty() {
            return OvsdbValue::Set(vec![]);
        }

        // Try to convert each item to an OvsdbAtom
        let mut atoms = Vec::with_capacity(self.len());
        for item in self {
            match item.to_ovsdb() {
                OvsdbValue::Atom(atom) => atoms.push(atom),
                _ => return OvsdbValue::Set(vec![]), // Invalid conversion, return empty set
            }
        }

        OvsdbValue::Set(atoms)
    }

    fn from_ovsdb(value: &OvsdbValue) -> Option<Self> {
        match value {
            OvsdbValue::Set(atoms) => {
                let mut result = Vec::with_capacity(atoms.len());
                for atom in atoms {
                    if let Some(item) = T::from_ovsdb(&OvsdbValue::Atom(atom.clone())) {
                        result.push(item);
                    } else {
                        return None;
                    }
                }
                Some(result)
            }
            // Handle single atom as a one-element set
            OvsdbValue::Atom(atom) => {
                T::from_ovsdb(&OvsdbValue::Atom(atom.clone())).map(|item| vec![item])
            }
            _ => None,
        }
    }
}

impl<K: OvsdbSerializable + ToString + Eq + std::hash::Hash, V: OvsdbSerializable> OvsdbSerializable
    for HashMap<K, V>
{
    fn to_ovsdb(&self) -> OvsdbValue {
        let mut pairs = Vec::with_capacity(self.len());

        for (key, value) in self {
            if let OvsdbValue::Atom(key_atom) = key.to_ovsdb() {
                if let OvsdbValue::Atom(value_atom) = value.to_ovsdb() {
                    pairs.push((key_atom, value_atom));
                    continue;
                }
            }
            return OvsdbValue::Map(vec![]);
        }

        OvsdbValue::Map(pairs)
    }

    fn from_ovsdb(value: &OvsdbValue) -> Option<Self> {
        match value {
            OvsdbValue::Map(map) => {
                let mut result = HashMap::with_capacity(map.len());

                for (key, val) in map {
                    if let Some(key_converted) = K::from_ovsdb(&OvsdbValue::Atom(key.clone())) {
                        if let Some(val_converted) = V::from_ovsdb(&OvsdbValue::Atom(val.clone())) {
                            result.insert(key_converted, val_converted);
                        } else {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }

                Some(result)
            }
            _ => None,
        }
    }
}

/// Custom serde serialization format for OvsdbValue
/// Implements the specific JSON format required by OVSDB
impl Serialize for OvsdbValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            OvsdbValue::Atom(atom) => atom.serialize(serializer),
            OvsdbValue::Set(set) => {
                if set.is_empty() {
                    let empty: Vec<String> = vec![];
                    empty.serialize(serializer)
                } else if set.len() == 1 {
                    set[0].serialize(serializer)
                } else {
                    let wrapper = ("set", set);
                    wrapper.serialize(serializer)
                }
            }
            OvsdbValue::Map(map) => {
                let pairs: Vec<[&OvsdbAtom; 2]> = map.iter().map(|(k, v)| [k, v]).collect();
                let wrapper = ("map", pairs);
                wrapper.serialize(serializer)
            }
        }
    }
}

impl Serialize for OvsdbAtom {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            OvsdbAtom::String(s) => s.serialize(serializer),
            OvsdbAtom::Integer(i) => i.serialize(serializer),
            OvsdbAtom::Real(r) => r.serialize(serializer),
            OvsdbAtom::Boolean(b) => b.serialize(serializer),
            OvsdbAtom::Uuid(uuid) => {
                let wrapper = ("uuid", uuid.to_string());
                wrapper.serialize(serializer)
            }
            OvsdbAtom::NamedUuid(name) => {
                let wrapper = ("named-uuid", name);
                wrapper.serialize(serializer)
            }
        }
    }
}

/// Extension trait for OvsdbSerializable to handle JSON conversion
pub trait OvsdbSerializableExt: OvsdbSerializable {
    fn to_ovsdb_json(&self) -> Option<serde_json::Value> {
        serde_json::to_value(self.to_ovsdb()).ok()
    }

    fn from_ovsdb_json(json: &serde_json::Value) -> Option<Self> {
        // Convert JSON to OvsdbValue
        let value = json_to_ovsdb_value(json)?;
        Self::from_ovsdb(&value)
    }
}

// Implement the extension trait for all types that implement OvsdbSerializable
impl<T: OvsdbSerializable> OvsdbSerializableExt for T {}

/// Helper function to extract a UUID from a JSON value
pub fn extract_uuid(value: &serde_json::Value) -> Option<Uuid> {
    if let serde_json::Value::Array(arr) = value {
        if arr.len() == 2 && arr[0] == "uuid" {
            if let serde_json::Value::String(uuid_str) = &arr[1] {
                return Uuid::parse_str(uuid_str).ok();
            }
        }
    }
    None
}

/// Convert a JSON value to an OvsdbValue
fn json_to_ovsdb_value(json: &serde_json::Value) -> Option<OvsdbValue> {
    match json {
        serde_json::Value::String(s) => Some(OvsdbValue::Atom(OvsdbAtom::String(s.clone()))),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(OvsdbValue::Atom(OvsdbAtom::Integer(i)))
            } else {
                n.as_f64().map(|f| OvsdbValue::Atom(OvsdbAtom::Real(f)))
            }
        }
        serde_json::Value::Bool(b) => Some(OvsdbValue::Atom(OvsdbAtom::Boolean(*b))),
        serde_json::Value::Array(arr) => {
            if arr.len() == 2 {
                if let serde_json::Value::String(tag) = &arr[0] {
                    match tag.as_str() {
                        "uuid" => {
                            if let serde_json::Value::String(uuid_str) = &arr[1] {
                                if let Ok(uuid) = Uuid::parse_str(uuid_str) {
                                    return Some(OvsdbValue::Atom(OvsdbAtom::Uuid(uuid)));
                                }
                            }
                        }
                        "named-uuid" => {
                            if let serde_json::Value::String(name) = &arr[1] {
                                return Some(OvsdbValue::Atom(OvsdbAtom::NamedUuid(name.clone())));
                            }
                        }
                        "set" => {
                            if let serde_json::Value::Array(elements) = &arr[1] {
                                let mut atoms = Vec::with_capacity(elements.len());
                                for elem in elements {
                                    if let Some(OvsdbValue::Atom(atom)) = json_to_ovsdb_value(elem)
                                    {
                                        atoms.push(atom);
                                    } else {
                                        return None;
                                    }
                                }
                                return Some(OvsdbValue::Set(atoms));
                            }
                        }
                        "map" => {
                            if let serde_json::Value::Array(pairs) = &arr[1] {
                                let mut map_pairs = Vec::with_capacity(pairs.len());
                                for pair in pairs {
                                    if let serde_json::Value::Array(kv) = pair {
                                        if kv.len() == 2 {
                                            if let (
                                                Some(OvsdbValue::Atom(key)),
                                                Some(OvsdbValue::Atom(value)),
                                            ) = (
                                                json_to_ovsdb_value(&kv[0]),
                                                json_to_ovsdb_value(&kv[1]),
                                            ) {
                                                map_pairs.push((key, value));
                                                continue;
                                            }
                                        }
                                    }
                                    return None;
                                }
                                return Some(OvsdbValue::Map(map_pairs));
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Empty array means empty set
            if arr.is_empty() {
                return Some(OvsdbValue::Set(vec![]));
            }

            None
        }
        serde_json::Value::Null => {
            // Null is represented as an empty set
            Some(OvsdbValue::Set(vec![]))
        }
        _ => None,
    }
}
