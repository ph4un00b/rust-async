use async_stream::stream;
use futures::pin_mut;
use std::{sync::Arc, thread, time::Duration};
use tokio::{sync::Notify, time::Instant};
use tokio_stream::StreamExt;

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

    notify.notified().await;
}

#[tokio::main]
async fn main() {
    use async_stream::stream;
    use std::time::Duration;

    let s = stream! {
        let mut when =  1;
        for _ in 0..3 {
            // let delay = Delay { when };
            // delay.await;
            delay(Duration::from_millis(1_000 * when)).await;
            yield when;
            when += 1;
        }
    };

    pin_mut!(s); //? needed for iteration

    while let Some(value) = s.next().await {
        println!("got {}", value);
    }
}
