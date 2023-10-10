use tokio_stream::StreamExt;

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

#[tokio::main]
async fn main() {
    let mut stream = tokio_stream::iter(&[1, 2, 3]);

    while let Some(v) = stream.next().await {
        delay(Duration::from_millis(1_000)).await;
        println!("GOT = {:?}", v);
    }
}
