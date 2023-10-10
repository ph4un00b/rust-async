use mini_redis::client;
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
    client.publish("numbers", "7".into()).await?;

    Ok(())
}

async fn subscribe() -> mini_redis::Result<()> {
    let client = client::connect("127.0.0.1:6379").await?;
    let subscriber = client.subscribe(vec!["numbers".to_string()]).await?;
    let stream_of_messages = subscriber
        //? @see adpters: https://docs.rs/tokio-stream/0.1.14/tokio_stream/trait.StreamExt.html
        .into_stream()
        // * We use the filter adapter to drop any message that does not match the predicate.
        .filter(|msg| match msg {
            Ok(msg) if msg.content.len() == 1 => true,
            _ => false,
        })
        .map(|msg| msg.unwrap().content)
        // * This adapter limits the stream to yield at most n messages.
        .take(3);

    tokio::pin!(stream_of_messages);
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
