use mini_redis::client;

/*
 * This does not compile because both tasks need to access the client somehow.
 *
 * As Client does not implement Copy,
 * it will not compile without some code to facilitate this sharing.
 *
 * Additionally, Client::set takes &mut self,
 * which means that exclusive access is required to call it.
 *
 * We could open a connection per task, but that is not ideal.
 *
 * We cannot use std::sync::Mutex as .await would need to be
 * called with the lock held.
 *
 * We could use tokio::sync::Mutex, but that would only allow a
 * single in-flight request.
 *
 * If the client implements pipelining, an async mutex
 * results in underutilizing the connection.
 */
#[tokio::main]
async fn main() {
    // Establish a connection to the server
    let mut client = client::connect("127.0.0.1:6379").await.unwrap();

    // Spawn two tasks, one gets a key, the other sets a key
    let t1 = tokio::spawn(async {
        let res = client.get("foo").await;
    });

    let t2 = tokio::spawn(async {
        client.set("foo", "bar".into()).await;
    });

    t1.await.unwrap();
    t2.await.unwrap();
}
