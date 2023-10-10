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

#[tokio::main]
async fn main() {
    let (tx1, mut rx1) = mpsc::channel(128);
    let (tx2, mut rx2) = mpsc::channel(128);
    let (tx3, mut rx3) = mpsc::channel(128);

    tokio::spawn(async move {
        let _ = tx1.send("one").await;
        delay(Duration::from_millis(1_000)).await;
        let _ = tx2.send("two").await;
        delay(Duration::from_millis(1_000)).await;
        let _ = tx3.send("3").await;
        delay(Duration::from_millis(1_000)).await;
    });

    loop {
        /*
         * This example selects over the three channel receivers.
         *
         * When a message is received on any channel, it is written to STDOUT.
         * When a channel is closed, recv() returns with None.
         * By using pattern matching, the select!
         * macro continues waiting on the remaining channels.
         * When all channels are closed,
         *
         * the else branch is evaluated and the loop is terminated.
         */
        let msg = tokio::select! {
            Some(msg) = rx1.recv() => msg,
            Some(msg) = rx2.recv() => msg,
            Some(msg) = rx3.recv() => msg,
            else => { break }
            /*
             * The select! macro randomly picks branches to check
             * first for readiness.
             *
             * When multiple channels have pending values, a
             * random channel will be picked to receive from.
             *
             * This is to handle the case where the receive loop
             * processes messages slower than they are pushed into the channels,
             * meaning that the channels start to fill up.
             *
             * If select! did not randomly pick a branch to check first,
             * on each iteration of the loop, rx1 would be checked first.
             * If rx1 always contained a new message, the remaining channels
             * would never be checked.
             *
             * If when select! is evaluated, multiple channels have
             * pending messages, only one channel has a value popped.
             *
             * All other channels remain untouched, and their messages
             * stay in those channels until the next loop iteration.
             * No messages are lost.
             */
        };

        println!("Got {:?}", msg);
    }

    println!("All channels have been closed.");
}
