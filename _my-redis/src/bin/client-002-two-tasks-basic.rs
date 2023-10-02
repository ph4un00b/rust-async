use mini_redis::client;

/*
 * Message passing pattern❗
 *
 * Concurrency and queuing must be explicitly introduced.
 * Ways to do this include:
 *
 * tokio::spawn | select! | join! | mpsc::channel
 *
 * When doing so, take care to ensure the total amount of concurrency is bounded.
 * For example, when writing a TCP accept loop, ensure that the
 * total number of open sockets is bounded.
 * When using mpsc::channel, pick a manageable channel capacity.
 * Specific bound values will be application specific.
 *
 * Taking care and picking good bounds is a big part of
 * writing reliable Tokio applications.
 */

use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() {
    /*
     * The mpsc channel is used to send commands to the task managing
     * the redis connection.
     *
     * The multi-producer capability allows messages to be sent
     * from many tasks. Creating the channel returns two values,
     * a sender and a receiver. The two handles are used separately.
     * They may be moved to different tasks.
     */
    //? Create a new channel with a capacity of at most 32.
    //? If messages are sent faster than they are received,
    //? the channel will store them. Once the 32 messages are
    //? stored in the channel, calling send(...).await will go
    //? to sleep until a message has been removed by the receiver.
    const MAX_BUFFER: usize = 32;
    let (tx, mut rx) = channel(MAX_BUFFER);
    let tx2 = tx.clone();
    // Establish a connection to the server
    let mut _client = client::connect("127.0.0.1:6379").await.unwrap();

    // Spawn two tasks, one gets a key, the other sets a key
    tokio::spawn(async move { tx.send("sending from first handle").await });

    tokio::spawn(async move { tx2.send("sending from second handle").await });

    /*
     * Both messages are sent to the single Receiver handle.
     * ❌ It is not possible to clone the receiver of an mpsc channel.
     */
    while let Some(message) = rx.recv().await {
        println!("GOT = {}", message);
    }
}
