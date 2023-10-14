use tokio::sync::{mpsc, oneshot};

/*
 * The basic idea behind an actor is to spawn a self-contained task
 * that performs some job independently of other parts of the program.
 *
 * Typically these actors communicate with the rest of the program
 * through the use of message passing channels.
 *
 * Since each actor runs independently, programs designed using
 * them are naturally parallel.
 *
 * @see https://ryhl.io/blog/actors-with-tokio/
 *
 * - what actors can't keep up❗
 * - ❌ backpressure: in bound channel, if you can send it will sleep until it can
 * maybe you can send a failure to the user in the mean time
 * - ❌ drop messages: tokio#broadcast channel handles this.
 * - ❌ kill the actor: if you use try send and it can't
 * it will send you back an error❗
 * - ❌ memory leak: on unbounded channels, if the messages keep increasing
 * it will happen eventually
 *
 * - a disadvantage of using actors:
 * - allocations: prefer mutex for less annotations but not fancy features.
 *
 * fancy feature:
 * - ✅ background tasks
 * - ✅ no need to wait for task completion
 */
struct Zeus {
    ear: mpsc::Receiver<ActorMessage>,
    next_id: u32,
}

/*
 * NO lifetimes in your commands❗
 *
 * - it is an smell
 * hence the message should be destroyed first
 * if you you will be pointing to something that does not exist❗
 */
enum ActorMessage {
    // * you don't need to wait for a response❗
    Get { key: String },
    GetUniqueId { respond_to: oneshot::Sender<u32> },
}

impl Zeus {
    fn new(rx: mpsc::Receiver<ActorMessage>) -> Self {
        Zeus {
            ear: rx,
            next_id: 0,
        }
    }
    fn handle_message(&mut self, msg: ActorMessage) {
        match msg {
            /*
             * it is ok to put many function boundary if
             * many commands are needed❗
             */
            ActorMessage::GetUniqueId { respond_to } => {
                self.next_id += 1;
                //? The `let _ =` ignores any errors when sending.
                //?
                //? This can happen if the `select!` macro is used
                //? to cancel waiting for the response.
                let _ = respond_to.send(self.next_id);
            }
            ActorMessage::Get { key } => todo!(),
        }
    }
}

/*
 * this could be the struct fields as parameter
 *
 * - the point is to protect the i/o resource
 * - meaning ensure nobody can touch the i/o resource❗
 */
async fn process_actions(mut actor: Zeus) {
    /*
     * When all senders to the receiver have been dropped,
     * we know that we will never receive another message
     * and can therefore shut down the actor.
     *
     * When this happens, the call to .recv() returns None,
     * and since it does not match the pattern Some(msg),
     * the while loop exits and the function returns.
     */
    while let Some(msg) = actor.ear.recv().await {
        // * function boundary, it is fine to inline it
        actor.handle_message(msg);
    }
}

/*
 * Now that we have the actor itself, we also need a handle to the actor.
 *
 * A handle is an object that other pieces of code can use to talk to the actor,
 * and is also what keeps the actor alive.
 */
#[derive(Clone)]
pub struct Hermes {
    mouth: mpsc::Sender<ActorMessage>,
}

impl Hermes {
    pub fn new() -> Self {
        /*
         * multiple-producer, single-consumer channel.
         *
         * Since the channel allows multiple producers,
         * we can freely clone our handle to the actor,
         * allowing us to talk to it from multiple places.
         *
         * You should still make sure to use a bounded channel
         * so that the number of messages waiting in the channel
         * don't grow without bound.
         * In some cases this will mean that sending still needs
         * to be an async function to handle the cases where
         * the send operation needs to wait for more space in the channel.
         *
         * To avoid such a deadlock,
         * you must make sure that there are no cycles of channels
         * with bounded capacity.
         *
         * The reason for this is that the send method on a
         * bounded channel does not return immediately.
         *
         * Channels whose send method always returns immediately
         * do not count in this kind of cycle, as you cannot deadlock on such a send.
         */
        // todo: cause cycle @see https://ryhl.io/blog/actors-with-tokio/
        // todo: cause deadlocks @see https://ryhl.io/blog/actors-with-tokio/
        // * if you spawn an actor in a tokio#spawn_blocking can cause a deadloack❗
        // * tokio#spawn_blocking: for tasks that will exit on their own.
        // * in a chat actor <-> client actor you can have backpressure on both
        // * sides hance cause a deadlock❗
        // * on solution is to use an unbounded channel (never sleep)
        // * on solution is to use try_send (aka kill the actor)
        // * on solution is to use oneshoot channel
        // * on solution is to use broadcast channel
        let (tx, rx) = mpsc::channel(8);
        let actor = Zeus::new(rx);

        //? T: Future + Send + 'static, -> does not mean living forever❗
        //? ❌ than would imply memory leak
        //? ✅ has not lifetime annotation shorter than 'static
        // todo deep more on 'static
        //? T::Output: Send + 'static,
        tokio::spawn(process_actions(actor));

        Self { mouth: tx }
    }

    pub async fn get_unique_id(&self) -> u32 {
        let (send, recv) = oneshot::channel();
        let msg = ActorMessage::GetUniqueId { respond_to: send };
        //? Ignore send errors. If this send fails, so does the
        //? recv.await below. There's no reason to check for the
        //? same failure twice.
        /*
         * When dealing with channels, not all errors are fatal.
         * Because of this, the example sometimes uses let _ = to ignore errors.
         * Generally a send operation on a channel fails if the receiver has been dropped
         */
        let _ = self.mouth.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }
}

impl Default for Hermes {
    fn default() -> Self {
        Self::new()
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
