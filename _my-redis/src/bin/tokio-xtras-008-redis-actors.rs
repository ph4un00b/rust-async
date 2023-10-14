use tokio::sync::{mpsc, oneshot};

struct Zeus {
    ear: mpsc::Receiver<Command>,
    next_id: u32,
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

impl Zeus {
    fn new(rx: mpsc::Receiver<Command>) -> Self {
        Zeus {
            ear: rx,
            next_id: 0,
        }
    }
    async fn handle_command(&mut self, cmd: Command) {
        println!("process commands");
        //? Establish a connection to the server
        let mut client = match mini_redis::client::connect("127.0.0.1:6379").await {
            Ok(c) => c,
            Err(e) => return eprintln!("{e}"),
        };
        use Command::*;
        match cmd {
            Get { key } => {
                self.next_id += 1;
                println!("manager GET {key}, {}", self.next_id);
            }
            GetOneShoot { key, resp } => {
                let res = client.get(&key).await;
                println!("GET processed");
                let _ = resp.send(res);
            }
            _ => {
                eprintln!("should use one shoot channel");
            }
        }
    }
}

async fn process_actions(mut god: Zeus) {
    println!("process commands");
    /*
     * When all senders to the receiver have been dropped,
     * we know that we will never receive another message
     * and can therefore shut down the actor.
     *
     * When this happens, the call to .recv() returns None,
     * and since it does not match the pattern Some(msg),
     * the while loop exits and the function returns.
     */
    while let Some(cmd) = god.ear.recv().await {
        god.handle_command(cmd).await;
    }
}

#[derive(Clone)]
pub struct Hermes {
    mouth: mpsc::Sender<Command>,
}

impl Hermes {
    pub fn blocking_new() -> Self {
        let (tx, rx) = mpsc::channel(8);

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let goddess = Zeus::new(rx);

        std::thread::spawn(move || {
            runtime.block_on(async {
                let manager = tokio::spawn(process_actions(goddess));
                // let manager = tokio::spawn(handle(rx));
                match manager.await {
                    Ok(_) => {
                        println!("manager: ok")
                    }
                    Err(e) => eprintln!("manager: {e}"),
                };
            })
        });

        Self { mouth: tx }
    }

    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(8);
        let god = Zeus::new(rx);
        tokio::spawn(process_actions(god));

        Self { mouth: tx }
    }

    pub async fn get(&self, key: String) -> u32 {
        let (_tx, rx) = oneshot::channel();
        let msg = Command::Get { key };
        //? Ignore send errors. If this send fails, so does the
        //? recv.await below. There's no reason to check for the
        //? same failure twice.
        let _ = self.mouth.send(msg).await;
        rx.await.expect("Actor task has been killed")
    }

    pub fn blocking_cmd(&self, cmd: Command) {
        match self.mouth.blocking_send(cmd) {
            Ok(()) => {
                println!("spawned task!")
            }
            Err(_) => panic!("The shared runtime has shut down."),
        }
    }

    pub fn blocking_get(&self, value: String) {
        let (resp_tx, resp_rx) = oneshot::channel();

        let cmd = Command::GetOneShoot {
            key: value,
            resp: resp_tx,
        };

        match self.mouth.blocking_send(cmd) {
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

impl Default for Hermes {
    fn default() -> Self {
        Self::new()
    }
}

use std::time::Duration;

fn main() {
    let messenger = Hermes::blocking_new();
    std::thread::sleep(Duration::from_millis(3750));
    messenger.blocking_cmd(Command::Get {
        key: "hi actors".into(),
    });
    std::thread::sleep(Duration::from_millis(3750));
    messenger.blocking_get("jamon".into());
    // std::thread::sleep(Duration::from_millis(3750));
}
