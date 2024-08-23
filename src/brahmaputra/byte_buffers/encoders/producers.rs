use std::fmt::Debug;
use std::sync::Arc;
use rust_decimal::prelude::ToPrimitive;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use crate::brahmaputra::byte_buffers::concrete_functions::byte_buffer::ByteBuff;
use crate::brahmaputra::byte_buffers::concrete_functions::enums::MessageCode;
use crate::brahmaputra::byte_buffers::concrete_functions::producers_objects::{ChannelWriter, pool_socket_reader, pool_socket_writer, Producer, socket_current_conn};
use crate::brahmaputra::byte_buffers::concrete_functions::select_partition::select_partition;

impl Producer {
    pub async fn connect_producer(&mut self) {

        // setting current conn to 0
        let _ = socket_current_conn.write().await.insert(0);
        
        // Connect to the server and build the connection pool
        let pool_size = self.pool.unwrap_or(1);
        for i in 0..pool_size {
            match TcpStream::connect(self.servers.to_string()).await {
                Ok(conn) => {

                    let (read_half, write_half) = tokio::io::split(conn);

                    pool_socket_writer.insert(i, Arc::new(RwLock::new(Some(write_half))));

                    pool_socket_reader.insert(i, Arc::new(RwLock::new(Some(read_half))));
                }
                Err(err) => {
                    println!("Failed to connect to server: {}", err);
                }
            }
        }

        // creating channel
        let (tx, mut rx) = mpsc::channel::<Box<Vec<u8>>>(self.max_buffer_size.unwrap_or(100000) as usize);
        let _ = ChannelWriter.write().await.insert(tx);
        let pool_clone = Arc::new(self.pool.unwrap());
        let pool_clone_copy = Arc::clone(&pool_clone);

        // starting reader channel
        tokio::spawn(async move{
            while let Some(total_buf) = rx.recv().await {

                let mut conn_number = socket_current_conn.read().await.as_ref().unwrap().to_i32().unwrap();

                if conn_number >= pool_clone_copy.to_i32().unwrap() - 1 {
                    conn_number = 0;
                } else {
                    conn_number = conn_number + 1;
                }

                if pool_socket_writer.contains_key(&conn_number){
                    if let socket = pool_socket_writer.get(&conn_number).unwrap(){
                        match  socket.write().await.as_mut(){
                            None => {}
                            Some(sock) => {
                                match sock.write_all(total_buf.as_slice()).await{
                                    Ok(_) => {
                                        match sock.flush().await {
                                            Err(err) => {
                                                println!("{:?}", err);
                                            }
                                            _ => {}
                                        }
                                    }
                                    Err(err) => {
                                        println!("{:?}", err);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        // receives the channels from socket
        tokio::spawn(async move{
            for i in 0..pool_size {
                let count = i;
                tokio::spawn(async move {
                    loop {
                        if pool_socket_reader.contains_key(&count){

                            if let socket = pool_socket_reader.get(&count).unwrap(){

                                let mut length_buf = [0u8; 8]; // Buffer to store incoming data

                                match socket.write().await.as_mut(){
                                    None => {}
                                    Some(sock) => {
                                        match sock.read_exact(&mut length_buf).await {

                                            Ok(_) => {

                                                // Convert the 8-byte array to an usize
                                                let total_msg_length = usize::from_be_bytes(length_buf);

                                                if total_msg_length > 0 {

                                                    // Create a buffer for the remaining part of the message
                                                    let mut total_buf = Box::new(vec![0u8; total_msg_length]);

                                                    // Read the exact message length data into the buffer
                                                    match sock.read_exact(&mut total_buf).await {
                                                        Ok(_) => {
                                                            // Process the full message
                                                            producer_decode_msg(total_buf).await;
                                                        }
                                                        Err(err) => {
                                                            println!("Error reading message data: {}", err);
                                                            //remove_socket_client(producer_id);
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to read from socket; err = {:?}", e);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }

                        }
                    }
                });
            }
        });
    }

    pub async fn push(&mut self, topic: String, key: String, msg: Vec<u8>) {
        // Encode the message using the producer_encode_msg_v1 method
        let message_byte = self.producer_encode_msg_v1(topic, key, msg).await;
        let _ = ChannelWriter.write().await.as_mut().unwrap().send(message_byte).await.is_ok();
    }

    async fn producer_encode_msg_v1(&self, topic: String, key: String, msg: Vec<u8>) -> Box<Vec<u8>> {
        let mut bb = ByteBuff {
            multiplier: 10000.0,
            endian: "big".to_string(),
            ..Default::default()
        };

        // into big endian format
        bb.init("big".to_string());

        // version number
        bb.put_string("V_1".to_string());

        // topic
        bb.put_string(topic);

        // message type either producer or consumer
        bb.put_string("P".to_string());

        // message code for producer
        bb.put_int(MessageCode::ProducerMsg as i32); // 100 for messages push

        // if there is any compression
        bb.put_string(self.compression_type.as_ref().unwrap().to_string());

        // acks
        bb.put_string(self.acks.as_ref().unwrap().to_string());

        // partition
        let partition = select_partition(key.to_string(), 5);

        // put partition
        bb.put_int(partition as i32);

        // put unique key
        bb.put_string(Uuid::new_v4().to_string());

        // key
        bb.put_string(key);

        // message
        bb.put(msg);

        // converting the total message into byte array
        let total_msg = bb.to_array();

        // wrapping the message into another byte array to get its total length
        let mut wrap_byte = ByteBuff {
            multiplier: 10000.0,
            endian: "big".to_string(),
            ..Default::default()
        };

        wrap_byte.put(total_msg);

        Box::new(wrap_byte.to_array())
    }
}

async fn producer_decode_msg(total_buf: Box<Vec<u8>>){

    let mut bb = Box::new(ByteBuff{
        multiplier: 10000.0,
        endian: "big".to_string(),
        ..Default::default()
    });

    bb.wrap(total_buf.to_vec());

    // deallocating the box
    drop(total_buf);

    // putting as P
    let client_type = bb.get_string();

    // putting Error Code
    let error_code = bb.get_int();

    // putting error message
    let error_msg = bb.get_string();

    // putting topic
    let topic = bb.get_string();

    // putting partition
    let partition = bb.get_int();

    // putting unique key
    let unique_key = bb.get_string();

    // putting key
    let key = bb.get_string();

    println!("{}", error_msg);
}
