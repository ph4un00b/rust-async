use std::{sync::Arc, thread, time::Duration};

use tokio::{sync::Notify, time::Instant};

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

async fn action() {
    println!("init action");
    delay(Duration::from_millis(2_000)).await;
    println!("ending action");
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(128);

    let operation = action();
    /*
     * the value being referenced must be pinned or implement Unpin.
     * If we remove the tokio::pin! line and try to compile,
     * we get the following error:
     * error[E0599]: no method named `poll` found for struct
     * @see https://doc.rust-lang.org/std/pin/index.html
     */
    tokio::pin!(operation);

    tokio::spawn(async move {
        println!("spawned task, will send");
        delay(Duration::from_millis(3_000)).await;
        println!("sending");
        let _ = tx.send(10).await;
    });

    /*
     * Then we call tokio::pin! on operation.
     * Inside the select! loop, instead of passing in operation,
     * we pass in &mut operation.
     *
     * The operation variable is tracking the in-flight asynchronous operation.
     *
     * Each iteration of the loop uses the same operation instead of issuing
     * a new call to action().
     */
    loop {
        tokio::select! {
            _ = &mut operation => {
                println!("break from operation");
                break;
            },
            Some(v) = rx.recv() => {
                if v % 2 == 0 {
                    println!("break from stream");
                    break;
                }
            }
        }
    }
}
