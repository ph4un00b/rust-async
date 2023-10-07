use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
use std::time::{Duration, Instant};

use crossbeam::channel;
use std::sync::Arc;

use std::sync::Mutex;

// type Task = Pin<Box<dyn Future<Output = ()> + Send>>;
pub struct Task {
    //? The `Mutex` is to make `Task` implement `Sync`. Only
    //? one thread accesses `future` at any given time. The
    //? `Mutex` is not required for correctness.❗

    //? Real Tokio
    //? does not use a mutex here, but real Tokio has
    //? more lines of code than can fit in a single tutorial
    //? page.
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    executor: channel::Sender<Arc<Task>>,
}

/*
 * Now, we need to hook our schedule function with std::task::Waker.
 *
 * The standard library provides a low-level API to do this using
 * manual vtable construction.
 * This strategy provides maximum flexibility to implementors,
 * @see https://doc.rust-lang.org/std/task/struct.RawWakerVTable.html
 * but requires a bunch of unsafe boilerplate code.
 *
 * we will use the ArcWake utility provided by the futures crate.
 * This allows us to implement a simple trait to expose our Task struct as a waker.
 */

impl futures::task::ArcWake for Task {
    /*
     * When the timer thread above calls waker.wake(),
     * the task is pushed into the channel.
     */
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}

impl Task {
    fn schedule(self: &Arc<Self>) {
        let _ = self.executor.send(self.clone());
    }

    fn poll(self: Arc<Self>) {
        //? Create a waker from the `Task` instance. This
        //? uses the `ArcWake` impl from above.
        let waker = futures::task::waker(self.clone());
        //* The waker is used to create a task::Context.
        //* That task::Context is passed to poll.
        let mut task_ctx = Context::from_waker(&waker);
        //? No other thread ever tries to lock the future
        let mut future = self.future.try_lock().unwrap();
        //? Poll the future
        let _ = future.as_mut().poll(&mut task_ctx);
    }

    //? Spawns a new task with the given future.
    //?
    //? Initializes a new Task harness containing the given future and pushes it
    //? onto `sender`. The receiver half of the channel will get the task and
    //? execute it.
    fn spawn<TFunc>(future: TFunc, sender: &channel::Sender<Arc<Task>>)
    where
        TFunc: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
    }
}

/*
* The updated Mini Tokio will use a channel to store scheduled tasks.
* Channels allow tasks to be queued for execution from any thread.
* Wakers must be Send and Sync, so we use the channel from the crossbeam crate,
* as the standard library channel is not Sync.
*
* Sync:  Types can be concurrently accessed through immutable references
*
* Send: Types that can be sent to a different thread
*/
struct MiniTokio {
    scheduled: channel::Receiver<Arc<Task>>,
    sender: channel::Sender<Arc<Task>>,
}

impl MiniTokio {
    /*
     * Additionally, the MiniTokio::new() and MiniTokio::spawn()
     * functions are adjusted to use a channel rather than a VecDeque.
     *
     * When new tasks are spawned, they are given a clone of the
     * sender-part of the channel, which the task can use to schedule
     * itself on the runtime.
     */
    fn new() -> MiniTokio {
        let (sender, scheduled) = channel::unbounded();
        MiniTokio { scheduled, sender }
    }

    //? Spawn a future onto the mini-tokio instance.
    //?
    //? The given future is wrapped with the `Task` harness and pushed into the
    //? `scheduled` queue. The future will be executed when `run` is called.
    fn spawn<TFunc>(&self, future: TFunc)
    where
        TFunc: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    /*
     * The function runs in a loop receiving scheduled tasks from the channel.
     * As tasks are pushed into the channel when they are woken,
     * these tasks are able to make progress when executed.
     */
    fn run(&self) {
        while let Ok(task) = self.scheduled.recv() {
            //* The Task::poll() function creates the waker
            //* using the ArcWake utility from the futures crate.
            task.poll();
        }
    }
}

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
        } else {
            //? Get a handle to the waker for the current task
            let waker = cx.waker().clone();
            let when = self.when;

            //? Spawn a timer thread.
            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }

                waker.wake();
            });
            Poll::Pending
        }
    }
}

fn main() {
    let mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async {
        let when = Instant::now() + Duration::from_millis(10);
        let future = Delay { when };

        let out = future.await;
        println!("out !{out}");
        assert_eq!(out, "done");
    });

    mini_tokio.run();
}
