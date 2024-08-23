use std::sync::Arc;
use dashmap::DashMap;
use lazy_static::lazy_static;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct Producer {
    pub max_buffer_size: Option<u64>,
    pub servers: String,
    pub message_timeout_ms: Option<u64>,
    pub delivery_timeout_ms: Option<u64>,
    pub batch_size: Option<u64>,
    pub compression_type: Option<String>,
    pub acks: Option<String>,
    pub retries: Option<u8>,
    pub retry_backoff_ms: Option<u64>,
    pub retry_backoff_max_ms: Option<u64>,
    pub reconnect_backoff_ms: Option<u64>,
    pub reconnect_backoff_max_ms: Option<u64>,
    pub socket_keepalive_enable: Option<bool>,
    pub pool: Option<i32>,
}

lazy_static! {
    pub static ref ChannelWriter: Arc<RwLock<Option<Sender<Box<Vec<u8>>>>>> = Arc::new(RwLock::new(None));
    pub static ref pool_socket_writer: DashMap<i32, Arc<RwLock<Option<WriteHalf<TcpStream>>>>> = DashMap::with_shard_amount(32);
    pub static ref pool_socket_reader: DashMap<i32, Arc<RwLock<Option<ReadHalf<TcpStream>>>>> = DashMap::with_shard_amount(32);
    pub static ref socket_current_conn: Arc<RwLock<Option<i32>>> = Arc::new(RwLock::new(Some(0)));
}