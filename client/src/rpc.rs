use crate::{
    schema::{DatabaseSchema, MonitorRequest, TableUpdate},
    transports::{ipc, tcp},
};
use jsonrpsee::{async_client::ClientBuilder, core::client::SubscriptionClientT, proc_macros::rpc};
use std::{collections::HashMap, path::Path};
use tokio::net::ToSocketAddrs;

#[rpc(client)]
pub trait Rpc {
    /// 4.1.1.  List Databases
    ///
    /// This operation retrieves an array whose elements are the names of the
    /// databases that can be accessed over this management protocol
    /// connection.
    #[method(name = "list_dbs")]
    async fn list_databases(&self) -> Result<Vec<String>, ErrorObjectOwned>;

    /// 4.1.2.  Get Schema
    ///
    /// This operation retrieves a <database-schema> that describes hosted
    /// database <db-name>.
    #[method(name = "get_schema")]
    async fn get_schema(&self, db_name: &str) -> Result<DatabaseSchema, ErrorObjectOwned>;

    /// 4.1.5.  Monitor
    ///
    /// The "monitor" request enables a client to replicate tables or subsets
    /// of tables within an OVSDB database by requesting notifications of
    /// changes to those tables and by receiving the complete initial state
    /// of a table or a subset of a table.
    #[method(name = "monitor")]
    async fn monitor(
        &self,
        db_name: &str,
        matcher: Option<&str>,
        requests: HashMap<String, MonitorRequest>,
    ) -> Result<TableUpdate<serde_json::Value>, ErrorObjectOwned>;

    /// 4.1.11.  Echo
    ///
    /// The "echo" method can be used by both clients and servers to verify
    /// the liveness of a database connection.  It MUST be implemented by
    /// both clients and servers.
    #[method(name = "echo")]
    async fn echo(
        &self,
        data: Vec<serde_json::Value>,
    ) -> Result<Vec<serde_json::Value>, ErrorObjectOwned>;
}

pub async fn connect_tcp(
    tcp: impl ToSocketAddrs,
) -> Result<impl SubscriptionClientT, std::io::Error> {
    let (sender, receiver) = tcp::connect(tcp).await?;

    Ok(ClientBuilder::default().build_with_tokio(sender, receiver))
}

pub async fn connect_unix(
    socket_path: impl AsRef<Path>,
) -> Result<impl SubscriptionClientT, std::io::Error> {
    let (sender, receiver) = ipc::connect(socket_path).await?;

    Ok(ClientBuilder::default().build_with_tokio(sender, receiver))
}
