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
    /*
     * it is important that the runtime is configured
     * to be a multi_thread runtime.
     *
     * If you change it to be a current_thread runtime,
     * you will find that the time consuming task finishes
     * before any of the background tasks start.
     */
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
        /*
         * The spawn method is also available on the Handle type.
         * The Handle type can be cloned to get many handles to a runtime,
         * and each Handle can be used to spawn new tasks on the runtime.
         */
        handles.push(runtime.spawn(my_big_task(i)));
    }

    //? Do something time-consuming while the background tasks execute.
    // * on single thread: this will block until finished❗
    std::thread::sleep(Duration::from_millis(6750));
    println!("Finished time-consuming task.");

    //? Wait for all of them to complete.
    for handle in handles {
        /*
         * As an example, this could be a good way of implementing
         * background network requests in a graphical application
         * because network requests are too time consuming to run
         * them on the main GUI thread.
         *
         * waits for the spawned tasks to finish by calling block_on on
         * the JoinHandle returned by the call to spawn,
         * but this isn't the only way to do it. Here are some alternatives:
         *
         * - Use a message passing channel such as tokio::sync::mpsc.
         *
         * - Modify a shared value protected by e.g. a Mutex.
         *  This can be a good approach for a progress bar in a GUI,
         * where the GUI reads the shared value every frame.
         */
        // todo: Use a message passing channel such as tokio::sync::mpsc.
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
