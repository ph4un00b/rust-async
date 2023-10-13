use bytes::Bytes;
use mini_redis::client;
use tokio::{
    spawn,
    sync::{
        mpsc::{channel, Receiver, Sender},
        oneshot,
    },
};

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
fn main() {
    /*
     * Message passing pattern❗
     */
    //? multi thread❗
    // let rt = tokio::runtime::Runtime::new().unwrap();
    //? single thread❗
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();

    rt.block_on(async {
        const MAX_BUFFER: usize = 32;
        let (tx, mut rx) = channel(MAX_BUFFER);
        let tx2 = tx.clone();
        let man = spawn(async move { process_commands(&mut rx).await });
        let get = spawn(async move { process_get(&tx).await });
        let set = spawn(async move { process_set(&tx2).await });
        get.await.unwrap();
        set.await.unwrap();
        match man.await {
            Ok(_) => (),
            Err(e) => eprintln!("{e}"),
        };
    });
}

async fn process_commands(rx: &mut Receiver<Command>) {
    //? Establish a connection to the server
    let mut client = match client::connect("127.0.0.1:6379").await {
        Ok(c) => c,
        Err(e) => return eprintln!("{e}"),
    };
    //? Start receiving messages
    use Command::*;
    while let Some(cmd) = rx.recv().await {
        match cmd {
            Get { key, resp } => {
                let res = client.get(&key).await;
                let _ = resp.send(res);
            }
            Set { key, val, resp } => {
                let res = client.set(&key, val).await;
                let _ = resp.send(res);
            }
        }
    }
}

async fn process_set(tx: &Sender<Command>) {
    let (resp_tx, resp_rx) = oneshot::channel();
    let cmd = Command::Set {
        key: "foo".to_string(),
        val: "bar".into(),
        resp: resp_tx,
    };

    if tx.send(cmd).await.is_err() {
        eprintln!("connection task shutdown");
        return;
    }

    let res = resp_rx.await;
    println!("GOT (Set) = {:?}", res);
}

async fn process_get(tx: &Sender<Command>) {
    let (resp_tx, resp_rx) = oneshot::channel();
    let cmd = Command::Get {
        key: "foo".to_string(),
        resp: resp_tx,
    };

    if tx.send(cmd).await.is_err() {
        eprintln!("connection task shutdown");
        return;
    }
    let res = resp_rx.await;
    println!("GOT (Get) = {:?}", res);
}
