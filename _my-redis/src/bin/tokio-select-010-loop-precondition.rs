use std::{sync::Arc, thread, time::Duration};

use tokio::{
    sync::{mpsc, Notify},
    time::Instant,
};

async fn delay(duration: Duration) {
    let when = Instant::now() + duration;
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    thread::spawn(move || {
        let now = Instant::now();
        if now < when {
            thread::sleep(when - now);
        }
        notify_clone.notify_one();
    });

    /*
     * If notify_one() is called before notified().await,
     * then the next call to notified().await will complete immediately,
     * consuming the permit.
     * Any subsequent calls to notified().await will wait for a new permit.
     *
     * If notify_one() is called multiple times before notified().await,
     * only a single permit is stored. The next call to notified().await
     * will complete immediately, but the one after will wait for a new permit.
     */
    notify.notified().await;
}

async fn action(input: Option<i32>) -> Option<String> {
    println!("init action: {input:?}");
    if let Some(4) = input {
        return Some("4".to_string());
    }
    delay(Duration::from_millis(2_000)).await;
    //? If the input is `None`, return `None`.
    //? This could also be written as `let i = input?;`
    let i = match input {
        Some(input) => input,
        None => {
            delay(Duration::from_millis(2_000)).await;
            println!("ending action NONE");
            return None;
        }
    };
    println!("ending action: {input:?}");
    delay(Duration::from_millis(2_000)).await;
    //? async logic here
    Some(format!("{i}"))
}

#[tokio::main]
async fn main() {
    // * A channel of i32 values.
    let (tx, mut rx) = mpsc::channel(128);

    let mut done = false;
    //* An async operation to perform on i32 values. */
    let operation = action(None);
    /*
     * The async fn is called outside of the loop and assigned to operation.
     * The operation variable is pinned.
     * The loop selects on both operation and the channel receiver.
     */
    tokio::pin!(operation);

    tokio::spawn(async move {
        delay(Duration::from_millis(1_000)).await;
        println!("send 1");
        let _ = tx.send(1).await;
        delay(Duration::from_millis(1_000)).await;
        println!("send 3");
        let _ = tx.send(3).await;
        delay(Duration::from_millis(1_000)).await;
        println!("send 2");
        let _ = tx.send(2).await;
        delay(Duration::from_millis(1_000)).await;
        println!("send 4");
        let _ = tx.send(4).await;
    });

    // todo "maybe made it a state machine❓"
    loop {
        // * Wait for an even number on the channel.

        // * Wait for the operation, but at the same time listen
        // *    for more even numbers on the channel.

        // * If a new even number is received before the existing operation completes,
        // *    abort the existing operation and start it over with the new even number.
        tokio::select! {
            /*
             * This is a branch precondition. (if !done)
             *
             * The done variable is used to track whether or not operation completed
             *
             * A select! branch may include a precondition.
             * This precondition is checked before select! awaits on the branch.
             *
             * If the condition evaluates to false then the branch is disabled.
             *
             * When operation completes, done is set to true.
             * The next loop iteration will disable the operation branch.
             * When an even message is received from the channel,
             * operation is reset and done is set to false.
             */
            res = &mut operation, if !done => {
                println!("ENTERED OPERATION: {res:?}");
                done = true;

                if let Some(v) = res {
                    println!("GOT = {}", v);
                    return;
                }
            }
            Some(v) = rx.recv() => {
                println!("ENTERED STREAM: {v}");
                if v % 2 == 0 {
                    // * Start the asynchronous operation using the even number as input.
                    //? `.set` is a method on `Pin`.
                    operation.set(action(Some(v)));
                    done = false;
                }
            }
        }
    }
}
