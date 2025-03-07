mod codec;
pub mod ipc;
pub mod tcp;

use bytes::BytesMut;
use futures_util::{Sink, SinkExt, Stream, stream::StreamExt};
use jsonrpsee::core::{
    async_trait,
    client::{ReceivedMessage, TransportReceiverT, TransportSenderT},
};
use serde_json::{Value, json};
use thiserror::Error;

#[derive(Debug, Error)]
enum TransportError {
    #[error("Connection closed.")]
    ConnectionClosed,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unkown error: {0}")]
    Unknown(String),
}

struct Sender<T: Send + Sink<BytesMut>> {
    inner: T,
}

#[async_trait]
impl<T: Send + Sink<BytesMut, Error = impl std::error::Error> + Unpin + 'static> TransportSenderT
    for Sender<T>
{
    type Error = TransportError;

    async fn send(&mut self, body: String) -> Result<(), Self::Error> {
        let mut message: Value =
            serde_json::from_str(&body).map_err(|e| TransportError::Unknown(e.to_string()))?;

        // NOTE(mnaser): In order to be able to use the subscription client, we need to
        //               drop the subscription message for the "update" method, as the
        //               remote doesn't support JSON-RPC 2.0.
        if message["method"] == json!("update") {
            return Ok(());
        }

        // NOTE(mnaser): jsonrpsee runs using JSON-RPC 2.0 only which the remote doesn't
        //               support, so we intercept the message, remove "jsonrpc" and then
        //               send the message.
        message.as_object_mut().map(|obj| obj.remove("jsonrpc"));

        // NOTE(mnaser): OVSDB expects all requests to have a "params" key, so we add an
        //               empty array if it doesn't exist.
        if !message.as_object().unwrap().contains_key("params") {
            message["params"] = json!([]);
        }

        self.inner
            .send(BytesMut::from(message.to_string().as_str()))
            .await
            .map_err(|e| TransportError::Unknown(e.to_string()))?;

        Ok(())
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        self.inner
            .close()
            .await
            .map_err(|e| TransportError::Unknown(e.to_string()))?;

        Ok(())
    }
}

struct Receiver<T: Send + Stream> {
    inner: T,
}

#[async_trait]
impl<T: Send + Stream<Item = Result<Value, std::io::Error>> + Unpin + 'static> TransportReceiverT
    for Receiver<T>
{
    type Error = TransportError;

    async fn receive(&mut self) -> Result<ReceivedMessage, Self::Error> {
        match self.inner.next().await {
            None => Err(TransportError::ConnectionClosed),
            Some(Ok(mut message)) => {
                // NOTE(mnaser): jsonrpsee runs using JSON-RPC 2.0 only which the remote doesn't
                //               support, so we intercept the message, add "jsonrpc" and then
                //               send the message.
                message
                    .as_object_mut()
                    .map(|obj| obj.insert("jsonrpc".to_string(), json!("2.0")));

                // NOTE(mnaser): jsonrpsee expects no error field if there is a result, due to the
                //               remote not supporting JSON-RPC 2.0, we need to remove the "error"
                //               field if there is a "result" field.
                if message.as_object().unwrap().contains_key("result") {
                    message.as_object_mut().map(|obj| obj.remove("error"));
                }

                // NOTE(mnaser): If a message comes in with it's "id" field set to null, then
                //               we remove it.
                if message.as_object().unwrap().contains_key("id") && message["id"] == json!(null) {
                    message.as_object_mut().map(|obj| obj.remove("id"));
                }

                Ok(ReceivedMessage::Bytes(message.to_string().into_bytes()))
            }
            Some(Err(e)) => Err(TransportError::Io(e)),
        }
    }
}
