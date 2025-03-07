use crate::transports::{Receiver, Sender, codec::JsonCodec};
use futures_util::stream::StreamExt;
use jsonrpsee::core::client::{TransportReceiverT, TransportSenderT};
use std::io::Error;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::codec::Framed;

pub async fn connect(
    socket: impl ToSocketAddrs,
) -> Result<(impl TransportSenderT + Send, impl TransportReceiverT + Send), Error> {
    let connection = TcpStream::connect(socket).await?;
    let (sink, stream) = Framed::new(connection, JsonCodec).split();

    let sender = Sender { inner: sink };
    let receiver = Receiver { inner: stream };

    Ok((sender, receiver))
}
