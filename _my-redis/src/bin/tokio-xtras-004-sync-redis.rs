use mini_redis::client;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct CommandSpawner {
    sender_channel: mpsc::Sender<Command>,
}

impl CommandSpawner {
    pub fn new() -> CommandSpawner {
        let (tx, mut rx) = mpsc::channel(16);
        let tx2 = tx.clone();
        // let tx3 = tx.clone();
        //? Build the runtime for the new thread.
        //?
        //? The runtime is created before spawning the thread
        //? to more cleanly forward errors if the `unwrap()`
        //? panics.
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        std::thread::spawn(move || {
            runtime.block_on(async {
                let manager = tokio::spawn(async move {
                    println!("process commands");
                    //? Establish a connection to the server
                    let mut client = match client::connect("127.0.0.1:6379").await {
                        Ok(c) => c,
                        Err(e) => return eprintln!("{e}"),
                    };
                    //? Start receiving messages
                    use Command::*;
                    while let Some(cmd) = rx.recv().await {
                        match cmd {
                            Get { key } => {
                                println!("manager GET {key}");
                            }
                            GetOneShoot { key, resp } => {
                                let res = client.get(&key).await;
                                println!("GET processed");
                                let _ = resp.send(res);
                            }
                            // SetOneShoot { key, val, resp } => {
                            //     let res = client.set(&key, val).await;
                            //     println!("SET processed");
                            //     let _ = resp.send(res);
                            // }
                            _ => {
                                eprintln!("should use one shoot channel");
                            }
                        }
                    }
                });

                match manager.await {
                    Ok(_) => (),
                    Err(e) => eprintln!("{e}"),
                };
            });
        });

        CommandSpawner {
            sender_channel: tx2,
        }
    }

    pub fn spawn_command(&self, cmd: Command) {
        /*
         * Blocking send to call outside of asynchronous contexts.
         * This method is intended for use cases where you are
         * sending from synchronous code to asynchronous code,
         */
        match self.sender_channel.blocking_send(cmd) {
            Ok(()) => {
                println!("spawned task!")
            }
            Err(_) => panic!("The shared runtime has shut down."),
        }
    }

    pub fn spawn_get(&self, value: String) {
        /*
         * Blocking send to call outside of asynchronous contexts.
         * This method is intended for use cases where you are
         * sending from synchronous code to asynchronous code,
         */
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::GetOneShoot {
            key: value,
            resp: resp_tx,
        };

        match self.sender_channel.blocking_send(cmd) {
            Ok(()) => {
                println!("spawned GET task!")
            }
            Err(_) => eprintln!("The shared runtime has shut down."),
        }

        match resp_rx.blocking_recv() {
            Ok(res) => {
                println!("GOT (Get) = {:?}", res);
            }
            Err(_) => eprintln!("err!"),
        };
    }
}

impl Default for CommandSpawner {
    fn default() -> Self {
        Self::new()
    }
}

use bytes::Bytes;
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[derive(Debug)]
pub enum Command {
    GetOneShoot {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    SetOneShoot {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
    Get {
        key: String,
    },
    Set {
        key: String,
        val: Bytes,
    },
}

fn main() {
    let spawner = CommandSpawner::new();
    std::thread::sleep(Duration::from_millis(3750));
    spawner.spawn_command(Command::Get { key: "hi".into() });
    std::thread::sleep(Duration::from_millis(3750));
    spawner.spawn_get("jamon".into());
    // std::thread::sleep(Duration::from_millis(3750));
}
