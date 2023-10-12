use tokio::time::{sleep, Duration};
/*
 * The above section explains the simplest way to implement a
 *  synchronous wrapper, but it is not the only way. The approaches are:
 *
 * - ❌ Create a Runtime and call block_on on the async code.
 *
 * - ✅ Create a Runtime and spawn things on it.
 *
 * - ❌ Run the Runtime in a separate thread and send messages to it.
 */
fn main() {
    // let runtime = Builder::new_multi_thread()
    //     .worker_threads(1)
    //     .enable_all()
    //     .build()
    //     .unwrap();

    let runtime = tokio::runtime::Builder::new_current_thread()
        /*
         * The enable_all call enables the IO and timer drivers on the Tokio runtime.
         * If they are not enabled, the runtime is unable to perform IO or timers.
         */
        .enable_all()
        .build()
        .unwrap();

    let mut handles = Vec::with_capacity(10);

    for i in 0..10 {
        handles.push(runtime.spawn(my_big_task(i)));
    }

    //? Do something time-consuming while the background tasks execute.
    // * on single thread: this will block until finished❗
    std::thread::sleep(Duration::from_millis(6750));
    println!("Finished time-consuming task.");

    //? Wait for all of them to complete.
    for handle in handles {
        //? The `spawn` method returns a `JoinHandle`. A `JoinHandle` is
        //? a future, so we can wait for it using `block_on`.
        runtime.block_on(handle).unwrap();
    }
}

async fn my_big_task(i: u64) {
    //? By subtracting, the tasks with larger values of i sleep for a
    //? shorter duration.
    let millis = 10_000 - 500 * i;
    println!("Task {} sleeping for {} ms.", i, millis);

    sleep(Duration::from_millis(millis)).await;

    println!("Task {} stopping.", i);
}
