use tokio::net::ToSocketAddrs;
use tokio::runtime::Runtime;

//? Established connection with a Redis server.
pub struct BlockingRedisClient {
    //? The asynchronous `Client`.
    inner: mini_redis::client::Client,
    //? A `current_thread` runtime for executing operations on the
    //? asynchronous client in a blocking manner.
    tokio: Runtime,
}

impl BlockingRedisClient {
    pub fn connect<T: ToSocketAddrs>(addr: T) -> mini_redis::Result<BlockingRedisClient> {
        /*
         * One important detail is the use of the current_thread runtime.
         *
         * Usually when using Tokio, you would be using the default multi_thread runtime,
         * which will spawn a bunch of background threads so it can efficiently
         * run many things at the same time.
         *
         * For our use-case, we are only going to be doing one thing at the time,
         * so we won't gain anything by running multiple threads.
         */
        /*
         * Because the current_thread runtime does not spawn threads,
         * it only operates when block_on is called.
         *
         * Once block_on returns, all spawned tasks on that runtime will
         * freeze until you call block_on again.
         * Use the multi_threaded runtime if spawned tasks must keep running
         * when not calling block_on.
         */
        let rt = tokio::runtime::Builder::new_current_thread()
            /*
             * The enable_all call enables the IO and timer drivers on the Tokio runtime.
             * If they are not enabled, the runtime is unable to perform IO or timers.
             */
            .enable_all()
            .build()?;

        let client = mini_redis::client::connect(addr);
        //? Call the asynchronous connect method using the runtime.
        let inner = rt.block_on(client)?;

        Ok(BlockingRedisClient { inner, tokio: rt })
    }
}

use bytes::Bytes;
use std::time::Duration;

impl BlockingRedisClient {
    pub fn get(&mut self, key: &str) -> mini_redis::Result<Option<Bytes>> {
        self.tokio.block_on(self.inner.get(key))
    }

    pub fn set(&mut self, key: &str, value: Bytes) -> mini_redis::Result<()> {
        self.tokio.block_on(self.inner.set(key, value))
    }

    pub fn set_expires(&mut self, key: &str, val: Bytes, exp: Duration) -> mini_redis::Result<()> {
        self.tokio.block_on(self.inner.set_expires(key, val, exp))
    }

    pub fn publish(&mut self, channel: &str, message: Bytes) -> mini_redis::Result<u64> {
        self.tokio.block_on(self.inner.publish(channel, message))
    }
}

/*
 * The above section explains the simplest way to implement a
 *  synchronous wrapper, but it is not the only way. The approaches are:
 *
 * - ✅ Create a Runtime and call block_on on the async code.
 *
 * - ❌ Create a Runtime and spawn things on it.
 *
 * - ❌ Run the Runtime in a separate thread and send messages to it.
 */
fn main() {
    let mut client = match BlockingRedisClient::connect("127.0.0.1:6379") {
        Ok(c) => {
            println!("connected");
            c
        }
        Err(e) => return eprintln!("{e}"),
    };

    let x = client.get("jamon");
    println!("get {x:?}");
    let x = client.set("jamon", "hola".into());
    println!("set {x:?}");
}
