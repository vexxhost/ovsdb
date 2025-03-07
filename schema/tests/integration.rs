use ovsdb_derive::ovsdb_object;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

#[ovsdb_object]
#[derive(Debug, PartialEq)]
pub struct NbGlobal {
    pub name: Option<String>,
    pub nb_cfg: Option<i64>,
    pub nb_cfg_timestamp: Option<i64>,
    pub sb_cfg: Option<i64>,
    pub sb_cfg_timestamp: Option<i64>,
    pub hv_cfg: Option<i64>,
    pub hv_cfg_timestamp: Option<i64>,
    pub external_ids: Option<HashMap<String, String>>,
    pub connections: Option<Vec<Uuid>>,
    pub ssl: Option<Vec<Uuid>>,
    pub options: Option<HashMap<String, String>>,
    pub ipsec: Option<bool>,

    // Required fields
    pub _uuid: Option<Uuid>,
    pub _version: Option<Uuid>,
}

#[test]
fn test_nb_global_deserialization() {
    // The provided JSON sample
    let json_str = r#"{
        "connections": ["uuid", "601c7161-97df-42ae-b377-3baf21830d8f"],
        "external_ids": ["map", [["test", "bara"]]],
        "hv_cfg": 0,
        "hv_cfg_timestamp": 0,
        "ipsec": false,
        "name": "global",
        "nb_cfg": 0,
        "nb_cfg_timestamp": 0,
        "options": ["map", [["name", "global"], ["northd-backoff-interval-ms", "300"], ["northd_probe_interval", "5000"]]],
        "sb_cfg": 0,
        "sb_cfg_timestamp": 0,
        "ssl": ["set", []]
    }"#;

    let json_value: Value = serde_json::from_str(json_str).unwrap();

    // Parse the JSON to our NbGlobal struct
    let nb_global =
        NbGlobal::from_map(&serde_json::from_value(json_value.clone()).unwrap()).unwrap();

    // Test individual fields
    assert_eq!(nb_global.name, Some("global".to_string()));
    assert_eq!(nb_global.ipsec, Some(false));
    assert_eq!(nb_global.hv_cfg, Some(0));
    assert_eq!(nb_global.hv_cfg_timestamp, Some(0));
    assert_eq!(nb_global.nb_cfg, Some(0));
    assert_eq!(nb_global.nb_cfg_timestamp, Some(0));
    assert_eq!(nb_global.sb_cfg, Some(0));
    assert_eq!(nb_global.sb_cfg_timestamp, Some(0));

    // Test UUID field
    let connection_uuid = Uuid::parse_str("601c7161-97df-42ae-b377-3baf21830d8f").unwrap();
    assert_eq!(nb_global.connections, Some(vec![connection_uuid]));

    // Test empty set
    assert_eq!(nb_global.ssl, Some(vec![]));

    // Test maps
    let expected_external_ids = {
        let mut map = HashMap::new();
        map.insert("test".to_string(), "bara".to_string());
        map
    };
    assert_eq!(nb_global.external_ids, Some(expected_external_ids));

    let expected_options = {
        let mut map = HashMap::new();
        map.insert("name".to_string(), "global".to_string());
        map.insert("northd-backoff-interval-ms".to_string(), "300".to_string());
        map.insert("northd_probe_interval".to_string(), "5000".to_string());
        map
    };
    assert_eq!(nb_global.options, Some(expected_options));
}

#[test]
fn test_nb_global_serialization() {
    // Create an NbGlobal object with the same values as the JSON sample
    let mut nb_global = NbGlobal::new();

    // Set scalar values
    nb_global.name = Some("global".to_string());
    nb_global.ipsec = Some(false);
    nb_global.hv_cfg = Some(0);
    nb_global.hv_cfg_timestamp = Some(0);
    nb_global.nb_cfg = Some(0);
    nb_global.nb_cfg_timestamp = Some(0);
    nb_global.sb_cfg = Some(0);
    nb_global.sb_cfg_timestamp = Some(0);

    // Set UUID connection
    let connection_uuid = Uuid::parse_str("601c7161-97df-42ae-b377-3baf21830d8f").unwrap();
    nb_global.connections = Some(vec![connection_uuid]);

    // Set empty SSL set
    nb_global.ssl = Some(vec![]);

    // Set maps
    let mut external_ids = HashMap::new();
    external_ids.insert("test".to_string(), "bara".to_string());
    nb_global.external_ids = Some(external_ids);

    let mut options = HashMap::new();
    options.insert("name".to_string(), "global".to_string());
    options.insert("northd-backoff-interval-ms".to_string(), "300".to_string());
    options.insert("northd_probe_interval".to_string(), "5000".to_string());
    nb_global.options = Some(options);

    // Serialize to JSON
    let serialized = nb_global.to_map();

    // Verify each field
    assert_eq!(serialized.get("name").unwrap().as_str().unwrap(), "global");
    assert!(!serialized.get("ipsec").unwrap().as_bool().unwrap());
    assert_eq!(serialized.get("hv_cfg").unwrap().as_i64().unwrap(), 0);

    // Test UUID serialization
    let connections_json = serialized.get("connections").unwrap();
    assert!(connections_json.is_array());
    let connections_array = connections_json.as_array().unwrap();
    assert_eq!(connections_array[0].as_str().unwrap(), "uuid");
    assert_eq!(
        connections_array[1].as_str().unwrap(),
        "601c7161-97df-42ae-b377-3baf21830d8f"
    );

    // Test empty set serialization
    let ssl_json = serialized.get("ssl").unwrap();
    assert!(ssl_json.is_array());
    assert_eq!(ssl_json.as_array().unwrap().len(), 0);

    // Test map serialization
    let external_ids_json = serialized.get("external_ids").unwrap();
    assert!(external_ids_json.is_array());
    assert_eq!(
        external_ids_json.as_array().unwrap()[0].as_str().unwrap(),
        "map"
    );

    let options_json = serialized.get("options").unwrap();
    assert!(options_json.is_array());
    assert_eq!(options_json.as_array().unwrap()[0].as_str().unwrap(), "map");
}

#[test]
fn test_round_trip() {
    // JSON string representing an NB_Global object
    let json_str = r#"{
        "connections": ["uuid", "601c7161-97df-42ae-b377-3baf21830d8f"],
        "external_ids": ["map", [["test", "bara"]]],
        "hv_cfg": 0,
        "hv_cfg_timestamp": 0,
        "ipsec": false,
        "name": "global",
        "nb_cfg": 0,
        "nb_cfg_timestamp": 0,
        "options": ["map", [["name", "global"], ["northd-backoff-interval-ms", "300"], ["northd_probe_interval", "5000"]]],
        "sb_cfg": 0,
        "sb_cfg_timestamp": 0,
        "ssl": ["set", []]
    }"#;

    // Deserialize from JSON string to NbGlobal object
    let json_value: Value = serde_json::from_str(json_str).unwrap();
    let nb_global = NbGlobal::from_map(&serde_json::from_value(json_value).unwrap()).unwrap();

    // Serialize back to JSON
    let serialized = serde_json::to_value(nb_global.to_map()).unwrap();

    // Deserialize again
    let nb_global2 = NbGlobal::from_map(&serde_json::from_value(serialized).unwrap()).unwrap();

    // The two objects should be equal
    assert_eq!(nb_global, nb_global2);
}

#[test]
fn test_handle_single_element_set() {
    // JSON with a single UUID in connections (no ["set", ...] wrapper)
    let json_str = r#"{
        "connections": ["uuid", "601c7161-97df-42ae-b377-3baf21830d8f"],
        "name": "global"
    }"#;

    let json_value: Value = serde_json::from_str(json_str).unwrap();
    let nb_global = NbGlobal::from_map(&serde_json::from_value(json_value).unwrap()).unwrap();

    // Should be parsed as a Vec with one element
    let connection_uuid = Uuid::parse_str("601c7161-97df-42ae-b377-3baf21830d8f").unwrap();
    assert_eq!(nb_global.connections, Some(vec![connection_uuid]));
}

#[test]
fn test_handle_multiple_element_set() {
    // JSON with multiple UUIDs in connections using ["set", [...]] wrapper
    let json_str = r#"{
        "connections": ["set", [
            ["uuid", "601c7161-97df-42ae-b377-3baf21830d8f"],
            ["uuid", "701c7161-97df-42ae-b377-3baf21830d8f"]
        ]],
        "name": "global"
    }"#;

    let json_value: Value = serde_json::from_str(json_str).unwrap();
    let nb_global = NbGlobal::from_map(&serde_json::from_value(json_value).unwrap()).unwrap();

    // Should be parsed as a Vec with two elements
    let uuid1 = Uuid::parse_str("601c7161-97df-42ae-b377-3baf21830d8f").unwrap();
    let uuid2 = Uuid::parse_str("701c7161-97df-42ae-b377-3baf21830d8f").unwrap();
    assert_eq!(nb_global.connections, Some(vec![uuid1, uuid2]));
}

#[test]
fn test_empty_set() {
    // JSON with empty set
    let json_str = r#"{
        "ssl": ["set", []],
        "name": "global"
    }"#;

    let json_value: Value = serde_json::from_str(json_str).unwrap();
    let nb_global = NbGlobal::from_map(&serde_json::from_value(json_value).unwrap()).unwrap();

    // Should be parsed as an empty Vec
    assert_eq!(nb_global.ssl, Some(vec![]));
}

#[test]
fn test_serialization_single_element_set() {
    let mut nb_global = NbGlobal::new();

    // Set single UUID connection
    let connection_uuid = Uuid::parse_str("601c7161-97df-42ae-b377-3baf21830d8f").unwrap();
    nb_global.connections = Some(vec![connection_uuid]);

    // Serialize to JSON
    let serialized = nb_global.to_map();
    let connections_json = serialized.get("connections").unwrap();

    // Should be serialized as ["uuid", "..."] (not wrapped in ["set", [...]])
    assert!(connections_json.is_array());
    let connections_array = connections_json.as_array().unwrap();
    assert_eq!(connections_array.len(), 2);
    assert_eq!(connections_array[0].as_str().unwrap(), "uuid");
}

#[test]
fn test_serialization_multiple_element_set() {
    let mut nb_global = NbGlobal::new();

    // Set multiple UUID connections
    let uuid1 = Uuid::parse_str("601c7161-97df-42ae-b377-3baf21830d8f").unwrap();
    let uuid2 = Uuid::parse_str("701c7161-97df-42ae-b377-3baf21830d8f").unwrap();
    nb_global.connections = Some(vec![uuid1, uuid2]);

    // Serialize to JSON
    let serialized = nb_global.to_map();
    let connections_json = serialized.get("connections").unwrap();

    // Should be serialized as ["set", [...]]
    assert!(connections_json.is_array());
    let connections_array = connections_json.as_array().unwrap();
    assert_eq!(connections_array[0].as_str().unwrap(), "set");
}
