use std::time::Duration;

use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Task {
    name: String,
    //? info that describes the task
}

async fn handle_task(task: Task) {
    println!("Got task {}", task.name);
}

#[derive(Clone)]
pub struct TaskSpawner {
    spawn: mpsc::Sender<Task>,
}
/*
 * The above section explains the simplest way to implement a
 *  synchronous wrapper, but it is not the only way. The approaches are:
 *
 * - ❌ Create a Runtime and call block_on on the async code.
 *
 * - ❌ Create a Runtime and spawn things on it.
 *
 * - ✅ Run the Runtime in a separate thread and send messages to it.
 */
impl TaskSpawner {
    pub fn new() -> TaskSpawner {
        //? Set up a channel for communicating.
        let (tx, mut rx) = mpsc::channel(16);

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
            println!("spawned thread");
            /*
             * This example could be configured in many ways.
             * @see: https://docs.rs/tokio/1.33.0/tokio/sync/struct.Semaphore.html
             * For instance, you could use a Semaphore to limit the
             * number of active tasks, or you could use a channel
             * in the opposite direction to send a response to the spawner.
             * When you spawn a runtime in this way, it is a type of actor.
             * @see https://ryhl.io/blog/actors-with-tokio/
             */
            runtime.block_on(async move {
                println!("running background runtime");
                while let Some(task) = rx.recv().await {
                    tokio::spawn(handle_task(task));
                }
                //? Once all senders have gone out of scope,
                //? the `.recv()` call returns None and it will
                //? exit from the while loop and shut down the
                //? thread.
            });
        });

        TaskSpawner { spawn: tx }
    }

    pub fn spawn_task(&self, task: Task) {
        match self.spawn.blocking_send(task) {
            Ok(()) => {
                println!("spawned task!")
            }
            Err(_) => panic!("The shared runtime has shut down."),
        }
    }
}

fn main() {
    let t = TaskSpawner::new();
    for i in 0..11 {
        std::thread::sleep(Duration::from_millis(750));
        t.spawn_task(Task {
            name: format!("jamon: {}", i),
        });
    }
}
