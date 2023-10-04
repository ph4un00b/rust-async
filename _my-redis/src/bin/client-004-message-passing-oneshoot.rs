use mini_redis::client;

/*
 * Message passing pattern❗
 */

use bytes::Bytes;
use tokio::sync::{mpsc, oneshot};
/*
 * The Sender half of the channel is included in the
 * command to the manager task.
 */
//? Provided by the requester and used by the manager task to send
//? the command response back to the requester.
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}
#[tokio::main]
async fn main() {
    const MAX_BUFFER: usize = 32;
    let (tx, mut rx) = mpsc::channel(MAX_BUFFER);
    let tx2 = tx.clone();

    let manager = tokio::spawn(async move {
        // Establish a connection to the server
        let mut client = match client::connect("127.0.0.1:6379").await {
            Ok(c) => c,
            Err(e) => return eprintln!("{e}"),
        };
        // Start receiving messages
        while let Some(cmd) = rx.recv().await {
            match cmd {
                /*
                 * update the manager task to send
                 * the response over the oneshot channel.
                 */
                Command::Get { key, resp } => {
                    let res = client.get(&key).await;
                    /*
                     * Calling send on oneshot::Sender completes
                     * immediately and does not require an .await.
                     * This is because send on a oneshot channel will always
                     * fail or succeed immediately without any form of waiting.
                     */
                    //? Ignore errors
                    let _ = resp.send(res);
                    /*
                     * In our scenario, the receiver cancelling interest
                     * is an acceptable event.
                     * The Err returned by resp.send(...) does not need
                     * to be handled.
                     */
                }
                Command::Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    //? Ignore errors
                    let _ = resp.send(res);
                }
            }
        }
    });

    let get = tokio::spawn(async move {
        /*
         * Unlike mpsc, no capacity is specified as the capacity
         * is always one 1️⃣❗. Additionally, neither handle can be cloned.
         */
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };

        if tx.send(cmd).await.is_err() {
            eprintln!("connection task shutdown");
            return;
        }

        //? Await the response
        let res = resp_rx.await;
        println!("GOT (Get) = {:?}", res);
    });

    let set = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: resp_tx,
        };

        if tx2.send(cmd).await.is_err() {
            eprintln!("connection task shutdown");
            return;
        }

        //? Await the response
        let res = resp_rx.await;
        println!("GOT (Set) = {:?}", res);
    });

    get.await.unwrap();
    set.await.unwrap();
    match manager.await {
        Ok(_) => (),
        Err(e) => eprintln!("{e}"),
    };
}
