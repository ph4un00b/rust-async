use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

/*
 * The values are not shared between connections.
 * If another socket connects and tries to GET the hello key,
 * it will not find anything.
 */
#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        //? A new task is spawned for each inbound socket. The socket is
        //? moved to the new task and processed there.
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    //? A hashmap is used to store data
    let mut in_memory_db = HashMap::new();

    // Connection, provided by `mini-redis`, handles parsing frames from
    // the socket
    let mut connection = Connection::new(socket);

    //? Use `read_frame` to receive a command from the connection.
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                println!("is a set mf!, {cmd:?}");
                //? The value is stored as `Vec<u8>`
                in_memory_db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                println!("is a get mf!, {cmd:?}");
                if let Some(value) = in_memory_db.get(cmd.key()) {
                    //? `Frame::Bulk` expects data to be of type `Bytes`. This
                    //? type will be covered later in the tutorial. For now,
                    //? `&Vec<u8>` is converted to `Bytes` using `into()`.
                    Frame::Bulk(value.clone().into())
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
