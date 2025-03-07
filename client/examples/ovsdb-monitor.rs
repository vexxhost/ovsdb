use jsonrpsee::core::client::{Subscription, SubscriptionClientT};
use ovsdb_client::{
    rpc::{self, RpcClient},
    schema::{MonitorRequest, UpdateNotification},
};
use std::collections::HashMap;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let socket_addr = "127.0.0.1:6641";
    let database = "OVN_Northbound";
    let table = "NB_Global";

    let client = rpc::connect_tcp(socket_addr).await?;

    // // 4.1.1.  List Databases
    let _databases = client.list_databases().await?;

    // // 4.1.2.  Get Schema
    let schema = client.get_schema(database).await?;
    let columns = schema
        .tables
        .get(table)
        .expect("table not found")
        .columns
        .keys()
        .cloned()
        .collect::<Vec<_>>();

    let mut requests = HashMap::new();
    requests.insert(
        table.to_owned(),
        MonitorRequest {
            columns: Some(columns),
            ..Default::default()
        },
    );

    let initial = client.monitor("OVN_Northbound", None, requests).await?;
    println!("Initial state: {:?}", initial);

    let mut stream: Subscription<UpdateNotification<serde_json::Value>> = client.subscribe_to_method("update").await?;

    while let Some(update) = stream.next().await {
        match update {
            Ok(update) => println!("Received update: {:?}", update),
            Err(e) => eprintln!("Error receiving update: {:?}", e),
        }
    }

    Ok(())
}
