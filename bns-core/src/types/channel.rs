use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug)]
pub enum Events {
    Null,
    ConnectFailed,
    SendMsg(String),
    ReceiveMsg(String),
}

#[async_trait(?Send)]
pub trait Channel {
    type Sender;
    type Receiver;

    fn new(buffer: usize) -> Self;
    fn sender(&self) -> Self::Sender;
    fn receiver(&self) -> Self::Receiver;

    async fn send(&self, e: Events) -> Result<()>;
    async fn recv(&self) -> ();
    async fn handler(&self, e: Events) -> ();
}
