use tokio::sync::{mpsc, oneshot};

struct MyBetterActor {
    receiver: mpsc::Receiver<ActorMessage>,
    next_id: u32,
}

enum ActorMessage {
    GetUniqueId { respond_to: oneshot::Sender<u32> },
}

impl MyBetterActor {
    /*
     * the run function must take ownership of
     * MyBetterActor object❗
     *
     * - no & on self
     *
     * - also can be an impl MyBetterActor {} block❗
     */
    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg);
        }
    }

    fn new(receiver: mpsc::Receiver<ActorMessage>) -> Self {
        Self {
            receiver,
            next_id: 0,
        }
    }

    fn handle_message(&mut self, msg: ActorMessage) {
        match msg {
            ActorMessage::GetUniqueId { respond_to } => {
                self.next_id += 1;
                let _ = respond_to.send(self.next_id);
            }
        }
    }
}

struct MyActorHandle {
    sender: mpsc::Sender<ActorMessage>,
}

impl MyActorHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(8);
        let mut actor = MyBetterActor::new(receiver);
        /*
         * This works identically to the top-level function.
         * Note that, strictly speaking, it is possible to write a
         * version where the tokio::spawn is inside run, but I don't recommend that approach.
         */
        tokio::spawn(async move { actor.run().await });

        Self { sender }
    }
}

fn main() {
    // let spawner = CommandSpawner::new();
    // std::thread::sleep(Duration::from_millis(3750));
    // spawner.spawn_command(Command::Get { key: "hi".into() });
    // std::thread::sleep(Duration::from_millis(3750));
    // spawner.spawn_get("jamon".into());
    // std::thread::sleep(Duration::from_millis(3750));
}
