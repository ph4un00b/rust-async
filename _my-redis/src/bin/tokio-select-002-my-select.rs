use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::oneshot;

struct MySelect {
    /*
     * The select! macro can handle more than two branches.
     * The current limit is 64 branches. Each branch is structured as:
     * <pattern> = <async expression> => <handler>,
     */
    receiver1: oneshot::Receiver<&'static str>,
    receiver2: oneshot::Receiver<&'static str>,
}

impl Future for MySelect {
    type Output = ();
    /*
     * This is a simplified version. In practice, select!
     * includes additional functionality like randomly selecting
     * the branch to poll first.
     */
    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        if let Poll::Ready(val) = Pin::new(&mut self.receiver1).poll(context) {
            println!("rx1 completed first with {:?}", val);
            return Poll::Ready(());
        }

        if let Poll::Ready(val) = Pin::new(&mut self.receiver2).poll(context) {
            println!("rx2 completed first with {:?}", val);
            return Poll::Ready(());
        }

        /*
         * When a future returns Poll::Pending,
         * it must ensure the waker is signalled at some point in the future.
         * Forgetting to do this results in the task hanging indefinitely.
         */
        Poll::Pending
    }
}

#[tokio::main]
async fn main() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();

    tokio::spawn(async {
        let _ = tx1.send("one");
    });

    tokio::spawn(async {
        let _ = tx2.send("two");
    });

    /*
     * After .await receives the output from a future,
     * the future is dropped. This results in the futures
     * for both branches to be dropped.
     */
    MySelect {
        receiver1: rx1,
        receiver2: rx2,
    }
    .await;
}
