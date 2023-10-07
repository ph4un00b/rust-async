use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};

use crossbeam::channel;
use std::sync::Arc;

use std::sync::Mutex;

// type Task = Pin<Box<dyn Future<Output = ()> + Send>>;
pub struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    executor: channel::Sender<Arc<Task>>,
}

impl futures::task::ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.schedule();
    }
}

impl Task {
    fn schedule(self: &Arc<Self>) {
        let _ = self.executor.send(self.clone());
    }

    fn poll(self: Arc<Self>) {
        let waker = futures::task::waker(self.clone());
        let mut task_ctx = Context::from_waker(&waker);
        let mut future = self.future.try_lock().unwrap();
        let _ = future.as_mut().poll(&mut task_ctx);
    }

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

struct MiniTokio {
    scheduled: channel::Receiver<Arc<Task>>,
    sender: channel::Sender<Arc<Task>>,
}

impl MiniTokio {
    fn new() -> MiniTokio {
        let (sender, scheduled) = channel::unbounded();
        MiniTokio { scheduled, sender }
    }

    fn spawn<TFunc>(&self, future: TFunc)
    where
        TFunc: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    fn run(&self) {
        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }
}

struct Delay {
    when: Instant,
    //* This is Some when we have spawned a thread, and None otherwise.
    waker: Option<Arc<Mutex<Waker>>>,
}

impl Future for Delay {
    type Output = &'static str;

    /*
     * It is a bit involved, but the idea is, on each call to poll,
     * the future checks if the supplied waker matches the previously
     * recorded waker.
     *
     * If the two wakers match, then there is nothing else to do.
     * If they do not match, then the recorded waker must be updated.
     */
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        //? First, if this is the first time the future is called, spawn the
        //? timer thread. If the timer thread is already running, ensure the
        //? stored `Waker` matches the current task's waker.
        if let Some(waker) = &self.waker {
            let mut waker = waker.lock().unwrap();

            //? Check if the stored waker matches the current task's waker.
            //? This is necessary as the `Delay` future instance may move to
            //? a different task between calls to `poll`. If this happens, the
            //? waker contained by the given `Context` will differ and we
            //? must update our stored waker to reflect this change.
            if !waker.will_wake(cx.waker()) {
                *waker = cx.waker().clone();
            }
        } else {
            let when = self.when;
            let waker = Arc::new(Mutex::new(cx.waker().clone()));
            self.waker = Some(waker.clone());

            //? This is the first time `poll` is called, spawn the timer thread.
            thread::spawn(move || {
                let now = Instant::now();

                if now < when {
                    thread::sleep(when - now);
                }

                //? The duration has elapsed. Notify the caller by invoking
                //? the waker.
                let waker = waker.lock().unwrap();
                waker.wake_by_ref();
            });
        }

        //? Once the waker is stored and the timer thread is started, it is
        //? time to check if the delay has completed. This is done by
        //? checking the current instant. If the duration has elapsed, then
        //? the future has completed and `Poll::Ready` is returned.
        if Instant::now() >= self.when {
            Poll::Ready("done")
        } else {
            //? The duration has not elapsed, the future has not completed so
            //? return `Poll::Pending`.
            //?
            //? The `Future` trait contract requires that when `Pending` is
            //? returned, the future ensures that the given waker is signalled
            //? once the future should be polled again. In our case, by
            //? returning `Pending` here, we are promising that we will
            //? invoke the given waker included in the `Context` argument
            //? once the requested duration has elapsed. We ensure this by
            //? spawning the timer thread above.
            //?
            //? If we forget to invoke the waker, the task will hang
            //? indefinitely.
            Poll::Pending
        }
    }
}

fn main() {
    let mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async {
        let when = Instant::now() + Duration::from_millis(10);
        let future = Delay { when, waker: None };

        let out = future.await;
        println!("out !{out}");
        assert_eq!(out, "done");
    });

    mini_tokio.run();
}
