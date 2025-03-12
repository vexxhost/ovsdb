use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug, Deserialize)]
pub struct DatabaseSchema {
    pub name: String,

    pub version: String,

    #[serde(rename = "cksum")]
    pub checksum: Option<String>,

    pub tables: HashMap<String, TableSchema>,
}

#[derive(Debug, Deserialize)]
pub struct TableSchema {
    pub columns: HashMap<String, ColumnSchema>,

    #[serde(rename = "maxRows")]
    pub max_rows: Option<u64>,

    #[serde(rename = "isRoot")]
    pub is_root: Option<bool>,

    #[serde(rename = "indexes")]
    pub indexes: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Deserialize)]
pub struct ColumnSchema {
    pub r#type: serde_json::Value,

    #[serde(rename = "ephemeral")]
    pub ephemeral: Option<bool>,

    #[serde(rename = "mutable")]
    pub mutable: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MonitorRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub select: Option<MonitorRequestSelect>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MonitorRequestSelect {
    initial: Option<bool>,
    insert: Option<bool>,
    delete: Option<bool>,
    modify: Option<bool>,
}

pub type TableUpdate<T> = HashMap<String, TableUpdateRows<T>>;
pub type TableUpdateRows<T> = HashMap<String, T>;

#[derive(Debug, Deserialize)]
pub struct RowUpdate<T> {
    pub old: Option<T>,
    pub new: Option<T>,
}

#[derive(Debug)]
pub struct UpdateNotification<T> {
    pub id: Option<String>,
    pub message: TableUpdate<T>,
}

impl<'de, T> Deserialize<'de> for UpdateNotification<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Define a visitor that carries a PhantomData for T.
        struct UpdateNotificationVisitor<T> {
            marker: PhantomData<T>,
        }

        impl<'de, T> Visitor<'de> for UpdateNotificationVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = UpdateNotification<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter
                    .write_str("an array with two elements: Option<String> and a TableUpdate<T>")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let id: Option<String> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let message: TableUpdate<T> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                Ok(UpdateNotification { id, message })
            }
        }

        // Start deserializing using the visitor.
        deserializer.deserialize_seq(UpdateNotificationVisitor {
            marker: PhantomData,
        })
    }
}
