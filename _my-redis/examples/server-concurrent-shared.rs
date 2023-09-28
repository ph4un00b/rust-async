use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type MemoryDB = Arc<Mutex<HashMap<String, Bytes>>>;
/*
 * By default, the Tokio runtime uses a multi-threaded scheduler
 */
#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    // * Note that std::sync::Mutex and not tokio::sync::Mutex is used to guard the HashMap
    // * Using a blocking mutex to guard short critical sections is an acceptable strategy when contention is minimal.
    let in_memory_db = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        //? A new task is spawned for each inbound socket. The socket is
        //? moved to the new task and processed there.
        //? Clone the handle to the hash map.
        let db = in_memory_db.clone();
        println!("Accepted");
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, shared_db: MemoryDB) {
    use mini_redis::Command::{self, Get, Set};

    // Connection, provided by `mini-redis`, handles parsing frames from
    // the socket
    let mut connection = Connection::new(socket);

    //? Use `read_frame` to receive a command from the connection.
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let mut db = shared_db.lock().unwrap();
                println!("is a set mf!, {cmd:?}");
                //? The value is stored as `BYTES❗`
                db.insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let db = shared_db.lock().unwrap();
                println!("is a get mf!, {cmd:?}");
                if let Some(value) = db.get(cmd.key()) {
                    //? `Frame::Bulk` expects data to be of type `Bytes`. This
                    //? type will be covered later in the tutorial.
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // Write the response to the client
        connection.write_frame(&response).await.unwrap();
    }
}
