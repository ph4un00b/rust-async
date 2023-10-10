use mini_redis::client;
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

async fn publish() -> mini_redis::Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;
    //? Publish some data
    delay(Duration::from_millis(1_000)).await;
    client.publish("numbers", "1".into()).await?;
    delay(Duration::from_millis(1_000)).await;
    client.publish("numbers_2", "two".into()).await?;
    delay(Duration::from_millis(1_000)).await;
    client.publish("numbers", "3".into()).await?;
    delay(Duration::from_millis(1_000)).await;
    client.publish("numbers_2", "four".into()).await?;
    delay(Duration::from_millis(1_000)).await;
    client.publish("numbers", "five".into()).await?;
    delay(Duration::from_millis(1_000)).await;
    client.publish("numbers_2", "6".into()).await?;
    delay(Duration::from_millis(1_000)).await;
    Ok(())
}

async fn subscribe() -> mini_redis::Result<()> {
    let client = client::connect("127.0.0.1:6379").await?;
    let subscriber = client.subscribe(vec!["numbers".to_string()]).await?;
    let stream_of_messages = subscriber.into_stream();
    // * The into_stream() function returns a stream that is not pinned,
    // * we must explicitly pin it in order to iterate it.
    //? A Rust value is "pinned" when it can no longer be moved in memory.
    //? A key property of a pinned value is that pointers can be taken to
    //? the pinned data and the caller can be confident the pointer stays valid.
    //? This feature is used by async/await to support borrowing data across .await points.
    tokio::pin!(stream_of_messages);
    // *  Calling next() on a stream requires the stream to be pinned.
    while let Some(msg) = stream_of_messages.next().await {
        println!("got = {:?}", msg);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> mini_redis::Result<()> {
    tokio::spawn(async { publish().await });

    subscribe().await?;

    println!("DONE");

    Ok(())
}
