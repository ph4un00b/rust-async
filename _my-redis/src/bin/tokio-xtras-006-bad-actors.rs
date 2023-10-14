use tokio::sync::{mpsc, oneshot};

struct MyBadActor {
    receiver: mpsc::Receiver<ActorMessage>,
    sender: mpsc::Sender<ActorMessage>,
    next_id: u32,
}

enum ActorMessage {
    GetUniqueId { respond_to: oneshot::Sender<u32> },
}

impl MyBadActor {
    /*
     * many people find it more natural to define a run
     * method directly on the MyActor struct and spawn that.
     *
     * This certainly works too, but the reason I give an example
     * that uses a top-level function is that it more naturally
     * leads you towards the approach that doesn't give you lots of lifetime issues.
     *
     * - should take ownership
     * - no &
     */
    fn run(mut self) {
        /*
         * The two sources of trouble in this example are:
         *
         * - ❌ The tokio::spawn call is inside run.
         *
         * The first issue causes problems because the tokio::spawn
         * function requires the argument to be 'static.
         * This means that the new task must own everything inside it,
         * which is a problem because the method borrows self,
         * meaning that it is not able to give away ownership of self to the new task.
         */
        tokio::spawn(async move {
            while let Some(msg) = self.receiver.recv().await {
                self.handle_message(msg);
            }
        });
    }

    pub async fn get_unique_id(&self) -> u32 {
        let (send, recv) = oneshot::channel();
        let msg = ActorMessage::GetUniqueId { respond_to: send };

        // Ignore send errors. If this send fails, so does the
        // recv.await below. There's no reason to check for the
        // same failure twice.
        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }

    /*
     * The two sources of trouble in this example are:
     *
     * - ❌ The actor and the handle are the same struct.
     *
     * The second issue causes problems because Rust enforces the single-ownership principle.
     * If you combine both the actor and the handle into a single struct,
     * you are (at least from the compiler's perspective)
     * giving every handle access to the fields owned by the actor's task.
     *
     * E.g. the next_id integer should be owned only by the actor's task,
     * and should not be directly accessible from any of the handles.
     */
    fn handle_message(&mut self, msg: ActorMessage) {
        match msg {
            ActorMessage::GetUniqueId { respond_to } => {
                self.next_id += 1;
                //? The `let _ =` ignores any errors when sending.
                //?
                //? This can happen if the `select!` macro is used
                //? to cancel waiting for the response.
                let _ = respond_to.send(self.next_id);
            }
        }
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
