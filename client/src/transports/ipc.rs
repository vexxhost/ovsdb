use crate::transports::{Receiver, Sender, codec::JsonCodec};
use futures_util::stream::StreamExt;
use jsonrpsee::core::client::{TransportReceiverT, TransportSenderT};
use std::{io::Error, path::Path};
use tokio::net::UnixStream;
use tokio_util::codec::Framed;

pub async fn connect(
    socket: impl AsRef<Path>,
) -> Result<(impl TransportSenderT + Send, impl TransportReceiverT + Send), Error> {
    let connection = UnixStream::connect(socket).await?;
    let (sink, stream) = Framed::new(connection, JsonCodec).split();

    let sender = Sender { inner: sink };
    let receiver = Receiver { inner: stream };

    Ok((sender, receiver))
}
