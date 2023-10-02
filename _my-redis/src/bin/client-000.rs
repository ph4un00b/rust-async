#[allow(warnings)]
use mini_redis::{client, Result};
#[allow(warnings)]
use tokio::runtime;

use core::pin::pin;
use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake};
use std::thread::{self, Thread};

/// A waker that wakes up the current thread when called.
struct ThreadWaker(Thread);

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

#[allow(warnings)]
/// Run a future to completion on the current thread.
fn block_on<T>(fut: impl Future<Output = T>) -> T {
    // Pin the future so it can be polled.
    let mut fut = pin!(fut);

    // Create a new context to be passed to the future.
    let t = thread::current();
    let waker = Arc::new(ThreadWaker(t)).into();
    let mut cx = Context::from_waker(&waker);

    // Run the future to completion.
    //? @see https://doc.rust-lang.org/std/future/trait.Future.html#panics
    //? Once a future has completed (returned Ready from poll),
    //? calling its poll method again may panic, block forever,
    //? or cause other kinds of problems; the Future trait places no
    //? requirements on the effects of such a call.
    //? However, as the poll method is not marked unsafe,
    //? Rust’s usual rules apply: calls must never cause undefined behavior
    //? (memory corruption, incorrect use of unsafe functions, or the like),
    //? regardless of the future’s state.
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(res) => {
                println!("ready!");
                return res;
            }
            Poll::Pending => {
                println!("parkeando!");
                thread::park()
            }
        }
    }
}

// * POLL
// fn main() {
//     block_on(async {
//         println!("Hi from inside a future!");
//     });
// }

// * TOKIO TASKS
// #[tokio::main]
// async fn main() {
//     async fn my_background_op(id: i32) -> String {
//         let s = format!("Starting background task {}.", id);
//         println!("{}", s);
//         s
//     }

//     let ops = vec![1, 2, 3];
//     let mut tasks = Vec::with_capacity(ops.len());
//     for op in ops {
//         // This call will make them start running in the background
//         // immediately.
//         tasks.push(tokio::spawn(my_background_op(op)));
//     }

//     let mut outputs = Vec::with_capacity(tasks.len());
//     for task in tasks {
//         outputs.push(task.await.unwrap());
//     }
//     println!("{:?}", outputs);
// }

// * BASICS TOKIO FUNCS
// #[tokio::main]
// async fn main() -> Result<()> {
//     // use tokio::runtime::Builder;
//     //? build runtime
//     //? @see https://docs.rs/tokio/1.32.0/tokio/runtime/index.html#runtime-configurations
//     // TODO: learn how to config the runtime ❗
//     // let rt = runtime::Builder::new_current_thread().build()?;
//     // let runtime = Builder::new_multi_thread()
//     //     .worker_threads(4)
//     //     .thread_name("my-custom-name")
//     //     .thread_stack_size(3 * 1024 * 1024)
//     //     .build()
//     //     .unwrap();

//     use tokio::task;

//     // * the async way for https://doc.rust-lang.org/nightly/std/thread/fn.spawn.html
//     let join = task::spawn(async {
//         // ...
//         "hello world!"
//     });

//     // ...

//     // Await the result of the spawned task.
//     let result = join.await?;
//     println!("{result}");
//     assert_eq!(result, "hello world!");
//     //? ========================= ERR
//     // let join = task::spawn(async { panic!("something bad happened!") });

//     // assert!(join.await.is_err());
//     //? ======================== BLOCKING COMPUTATION PROPERLY HANDLED
//     // Initial input
//     let mut v = "Hello, ".to_string();
//     let res = task::spawn_blocking(move || {
//         // Stand-in for compute-heavy work or using synchronous APIs
//         v.push_str("world");
//         // Pass ownership of the value back to the asynchronous context
//         v
//     })
//     .await?;

//     // `res` is the value returned from the thread
//     println!("BLOCKING COMPUTATION PROPERLY HANDLED: {res}");
//     assert_eq!(res.as_str(), "Hello, world");
//     //? ========================= COMPUTATION HANDLED BY CHANNEL
//     use tokio::sync::mpsc;

//     let (tx, mut rx) = mpsc::channel(2);
//     let start = 5;
//     let worker = task::spawn_blocking(move || {
//         for x in 0..10 {
//             // Stand in for complex computation
//             tx.blocking_send(start + x).unwrap();
//         }
//     });

//     let mut acc = 0;
//     while let Some(v) = rx.recv().await {
//         acc += v;
//     }
//     println!("COMPUTATION HANDLED BY CHANNEL: {acc}");
//     assert_eq!(acc, 95);
//     worker.await.unwrap();
//     //? ========================== block_in_place
//     //? @see https://docs.rs/tokio/1.32.0/tokio/task/index.html#block_in_place
//     let result = task::block_in_place(|| {
//         // do some compute-heavy work or call synchronous code
//         "blocking completed"
//     });
//     println!("block_in_place: {result}");
//     assert_eq!(result, "blocking completed");
//     //? ============================== YIELD_NOW
//     let yield_now = async {
//         task::spawn(async {
//             // ...
//             println!("spawned task done!")
//         });

//         // Yield, allowing the newly-spawned task to execute first.
//         task::yield_now().await;
//         println!("main task done!");
//     };
//     yield_now.await;
//     // todo: COOPERATIVE SCHEDULING
//     //? @see https://docs.rs/tokio/1.32.0/tokio/task/index.html#cooperative-scheduling
//     Ok(())
// }

// * @see https://docs.rs/tokio/1.32.0/tokio/task/index.html#what-are-tasks
//? A task is a light weight, non-blocking unit of execution.
//? A task is similar to an OS thread, but rather than being
//?managed by the OS scheduler, they are managed by the Tokio runtime.
// * TOKIO JOINSET
// use tokio::task::JoinSet;

// #[tokio::main]
// async fn main() {
//     let mut set = JoinSet::new();

//     for i in 0..10 {
//         // * Available on tokio_unstable and crate feature tracing and crate feature rt only
//         // set.build_task().name("jamon-{i}").spawn(async move { i });
//         set.spawn(async move { i });
//     }

//     let mut seen = [false; 10];
//     while let Some(res) = set.join_next().await {
//         let idx = res.unwrap();
//         println!("{idx}!");
//         seen[idx] = true;
//     }

//     for i in 0..10 {
//         assert!(seen[i]);
//     }
// }

#[tokio::main]
async fn main() -> Result<()> {
    // Open a connection to the mini-redis address.
    let mut client = client::connect("127.0.0.1:6379").await?;

    // Set the key "hello" with value "world"
    client.set("hello", "world".into()).await?;

    // Get key "hello"
    let result = client.get("hello").await?;

    println!("jamon!; result={:?}", result);

    Ok(())
}

// * fn main() {
// *     let rt = tokio::runtime::Runtime::new().unwrap();
// *     rt.block_on(async {
// *         println!("hello");
// *     })
// * }

//*  equivalent code below❗

//? #[tokio::main]
//? async fn main() {
//?     println!("hello");
//? }
