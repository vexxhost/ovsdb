use ovsdb_derive::ovsdb_object;
use std::collections::HashMap;
use uuid::Uuid;

#[ovsdb_object]
#[derive(Debug, Clone, PartialEq)]
pub struct NbGlobal {
    pub name: String,
    pub nb_cfg: i64,
    pub nb_cfg_timestamp: i64,
    pub sb_cfg: i64,
    pub sb_cfg_timestamp: i64,
    pub hv_cfg: i64,
    pub hv_cfg_timestamp: i64,
    pub external_ids: HashMap<String, String>,
    pub connections: Vec<Uuid>,
    pub ssl: Vec<Uuid>,
    pub options: HashMap<String, String>,
    pub ipsec: bool,
}

fn main() {
    // Create a new NbGlobal instance
    let mut nb_global = NbGlobal::new();

    // Set some values
    nb_global.name = "global".to_string();
    nb_global.nb_cfg = 0;
    nb_global
        .external_ids
        .insert("test".to_string(), "value".to_string());

    // Convert to a HashMap for OVSDB serialization
    let map = nb_global.to_map();
    println!("{:?}", map);

    // Convert to JSON for sending to OVSDB
    let json = serde_json::to_string(&map).unwrap();
    println!("{}", json);

    // Simulate receiving JSON from OVSDB
    let received_map: HashMap<String, serde_json::Value> = serde_json::from_str(&json).unwrap();

    // Convert back to NbGlobal
    let parsed_nb_global = NbGlobal::from_map(&received_map).unwrap();
    println!("{:?}", parsed_nb_global);
}
