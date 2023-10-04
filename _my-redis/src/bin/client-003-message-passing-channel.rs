use mini_redis::client;

/*
 * Message passing pattern❗
 */

use bytes::Bytes;
use tokio::sync::mpsc::channel;

#[derive(Debug)]
enum Command {
    Get { key: String },
    Set { key: String, val: Bytes },
}
#[tokio::main]
async fn main() {
    const MAX_BUFFER: usize = 32;
    let (tx, mut rx) = channel(MAX_BUFFER);
    let tx2 = tx.clone();

    /*
     * Now, update the two tasks to send commands over the channel
     * instead of issuing them directly on the Redis connection.
     */
    //? Spawn two tasks, one gets a key, the other sets a key
    let t1 = tokio::spawn(async move {
        let cmd = Command::Get {
            key: "foo".to_string(),
        };

        tx.send(cmd).await.unwrap();
    });

    let t2 = tokio::spawn(async move {
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
        };

        tx2.send(cmd).await.unwrap();
    });

    let manager = tokio::spawn(async move {
        // Establish a connection to the server
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        // Start receiving messages
        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key } => {
                    let _ = client.get(&key).await;
                }
                Set { key, val } => {
                    let _ = client.set(&key, val).await;
                }
            }
        }
    });

    /*
     * At the bottom of the main function, we .await the join handles
     * to ensure the commands fully complete before the process exits.
     */
    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
