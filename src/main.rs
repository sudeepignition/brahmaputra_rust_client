mod brahmaputra;
use tokio::io::AsyncReadExt;
use tokio::task;
use crate::brahmaputra::byte_buffers::concrete_functions::producers_objects::Producer;

#[tokio::main]
async fn main() {

    let mut producer = Producer{
        servers: "localhost:9092".to_string(),
        acks: Some("all".to_string()),
        max_buffer_size: Some(1000000000),
        retries: Some(5),
        compression_type: Some("lz4".to_string()),
        pool: Some(100),
        ..Default::default()
    };

    producer.connect_producer().await;

    for i in 0..100000000{
        producer.push("loggers".to_string(), "sudeep key".to_string(), "hello sudeep".as_bytes().to_vec()).await;
    }

    loop{
        let result = task::spawn_blocking(|| {
            // Blocking operation here
            std::thread::sleep(std::time::Duration::from_secs(5));
            "Blocking operation completed"
        })
        .await
        .unwrap();
    }
}
