use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use futures::lock::Mutex;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio::time::Duration;

use crate::dht::Did;
use crate::dht::PeerRing;
use crate::ecc::SecretKey;
use crate::message::Message;
use crate::message::MessagePayload;
use crate::message::MessageVerificationExt;
use crate::session::SessionSk;
use crate::storage::MemStorage;
use crate::swarm::callback::SwarmCallback;
use crate::swarm::Swarm;
use crate::swarm::SwarmBuilder;

mod test_connection;
mod test_message_handler;
mod test_stabilization;

pub struct Node {
    pub swarm: Arc<Swarm>,
    message_rx: Mutex<mpsc::UnboundedReceiver<MessagePayload>>,
}

pub struct NodeCallback {
    message_tx: mpsc::UnboundedSender<MessagePayload>,
}

impl Node {
    pub fn new(swarm: Arc<Swarm>) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let callback = NodeCallback { message_tx };
        swarm.set_callback(Arc::new(callback)).unwrap();
        Self {
            swarm,
            message_rx: Mutex::new(message_rx),
        }
    }

    pub async fn listen_once(&self) -> Option<MessagePayload> {
        self.message_rx.lock().await.recv().await
    }

    pub fn did(&self) -> Did {
        self.swarm.did()
    }

    pub fn dht(&self) -> Arc<PeerRing> {
        self.swarm.dht().clone()
    }

    pub fn assert_transports(&self, addresses: Vec<Did>) {
        println!(
            "Check transport of {:?}: {:?} for addresses {:?}",
            self.did(),
            self.swarm.transport.get_connection_ids(),
            addresses
        );
        assert_eq!(
            self.swarm.transport.get_connections().len(),
            addresses.len()
        );
        for addr in addresses {
            assert!(self.swarm.transport.get_connection(addr).is_some());
        }
    }
}

#[async_trait]
impl SwarmCallback for NodeCallback {
    async fn on_validate(
        &self,
        payload: &MessagePayload,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Here we are using on_validate to record messages.
        // When on_validate return error, the message will be ignored, which is not on purpose.
        // To prevent returning errors when sending fails, we choose to panic instead.
        self.message_tx.send(payload.clone()).unwrap();
        Ok(())
    }
}

pub async fn prepare_node(key: SecretKey) -> Node {
    let stun = "stun://stun.l.google.com:19302";
    let storage = Box::new(MemStorage::new());

    let session_sk = SessionSk::new_with_seckey(&key).unwrap();
    let swarm = Arc::new(SwarmBuilder::new(0, stun, storage, session_sk).build());

    println!("key: {:?}", key.to_string());
    println!("did: {:?}", swarm.did());

    Node::new(swarm)
}

pub fn gen_pure_dht(did: Did) -> PeerRing {
    let storage = Box::new(MemStorage::new());
    PeerRing::new_with_storage(did, 3, storage)
}

pub fn gen_sorted_dht(s: usize) -> Vec<PeerRing> {
    let mut keys: Vec<crate::ecc::SecretKey> = vec![];
    for _i in 0..s {
        keys.push(crate::ecc::SecretKey::random());
    }
    keys.sort_by_key(|a| a.address());

    #[allow(clippy::needless_collect)]
    let dids: Vec<crate::dht::Did> = keys
        .iter()
        .map(|sk| crate::dht::Did::from(sk.address()))
        .collect();

    let mut iter = dids.into_iter();
    let mut ret: Vec<crate::dht::PeerRing> = vec![];
    for _ in 0..s {
        ret.push(crate::tests::default::gen_pure_dht(iter.next().unwrap()))
    }
    ret
}

pub async fn assert_no_more_msg(nodes: impl IntoIterator<Item = &Node>) {
    let did_names: DashMap<Did, String> = DashMap::new();
    let mut listeners = vec![];

    for (i, node) in nodes.into_iter().enumerate() {
        let name = format!("node{}", i + 1);
        did_names.insert(node.did(), name);

        listeners.push(async {
            let payload = node.listen_once().await.unwrap();
            format!(
                "{} should not receive any Msg, but got Msg {} -> {} [{} => {}] : {:?}",
                *did_names.get(&node.did()).unwrap(),
                *did_names.get(&payload.signer()).unwrap(),
                *did_names.get(&node.did()).unwrap(),
                *did_names.get(&payload.transaction.signer()).unwrap(),
                *did_names.get(&payload.transaction.destination).unwrap(),
                payload.transaction.data::<Message>().unwrap()
            )
        });
    }

    let mut listeners = FuturesUnordered::from_iter(listeners);

    tokio::select! {
        error_msg = listeners.next() => unreachable!("{}", error_msg.unwrap()),
        _ = sleep(Duration::from_secs(3)) => {}
    }
}

pub async fn wait_for_msgs(nodes: impl IntoIterator<Item = &Node>) {
    let did_names: DashMap<Did, String> = DashMap::new();
    let mut listeners = vec![];

    for (i, node) in nodes.into_iter().enumerate() {
        let name = format!("node{}", i + 1);
        did_names.insert(node.did(), name);

        listeners.push(async {
            loop {
                tokio::select! {
                    Some(payload) = node.listen_once() => {
                        println!(
                            "Msg {} -> {} [{} => {}] : {:?}",
                            *did_names.get(&payload.signer()).unwrap(),
                            *did_names.get(&node.did()).unwrap(),
                            *did_names.get(&payload.transaction.signer()).unwrap(),
                            *did_names.get(&payload.transaction.destination).unwrap(),
                            payload.transaction.data::<Message>().unwrap()
                        )
                    }
                    _ = sleep(Duration::from_secs(3)) => break
                }
            }
        });
    }

    futures::future::join_all(listeners).await;
}
