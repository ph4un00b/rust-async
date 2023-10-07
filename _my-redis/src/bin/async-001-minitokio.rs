use futures::task;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

type Task = Pin<Box<dyn Future<Output = ()> + Send>>;

struct MiniTokio {
    tasks: VecDeque<Task>,
}

impl MiniTokio {
    fn new() -> MiniTokio {
        MiniTokio {
            tasks: VecDeque::new(),
        }
    }

    //? Spawn a future onto the mini-tokio instance.
    fn spawn<TFunc>(&mut self, future: TFunc)
    where
        TFunc: Future<Output = ()> + Send + 'static,
    {
        self.tasks.push_back(Box::pin(future));
    }

    /*
     * The executor continuously loops all spawned futures and polls them.
     * Most of the time, the futures will not be ready to perform
     * more work and will return Poll::Pending again.
     * The process will burn CPU cycles and generally not be very efficient.
     */
    fn run(&mut self) {
        let waker = task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        while let Some(mut task) = self.tasks.pop_front() {
            if task.as_mut().poll(&mut cx).is_pending() {
                self.tasks.push_back(task);
            }
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
            //? Ignore this line for now.
            /*
             * This method #waker returns a Waker bound to the current task.
             *
             * Before returning Poll::Pending,
             * we called cx.waker().wake_by_ref().
             * This is to satisfy the future contract.\
             *
             * Because we didn't implement the timer thread yet,
             * we signalled the waker inline.
             * Doing so will result in the future being immediately re-scheduled,
             * executed again, and probably not be ready to complete.
             */
            cx.waker()
                /*
                 * Calling this method signals to the executor
                 * that the associated task should be scheduled for execution.
                 *
                 * Resources call wake() when they transition
                 * to a ready state to notify the executor
                 * that polling the task will be able to make progress.
                 */
                .wake_by_ref();
            Poll::Pending
        }
    }
}

fn main() {
    let mut mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async {
        let when = Instant::now() + Duration::from_millis(10);
        let future = Delay { when };

        let out = future.await;
        println!("out !{out}");
        assert_eq!(out, "done");
    });

    mini_tokio.run();
}
