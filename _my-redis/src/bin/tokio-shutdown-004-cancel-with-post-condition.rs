use tokio::sync::mpsc::{self, Sender};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    let (sender, mut rx) = mpsc::channel(1);

    for i in 0..10 {
        tokio::spawn(async_operation(i, sender.clone()));
    }

    /*
     * A very important detail is that the task waiting
     * for shutdown usually holds one of the senders.
     * When this is the case, you must make sure to drop
     * that sender before waiting for the channel to be closed.
     */
    //? Wait for the tasks to finish.
    //?
    //? We drop our sender first because the recv() call otherwise
    //? sleeps forever.
    drop(sender);

    //? When every sender has gone out of scope, the recv call
    //? will return with an error. We ignore the error.
    match rx.recv().await {
        Some(_) => println!("some"),
        None => println!("none"),
    }
}

async fn async_operation(i: u64, _sender: Sender<()>) {
    tokio::time::sleep(Duration::from_millis(1000 * i)).await;
    println!("Task {} shutting down.", i);

    //? sender goes out of scope ...
}
