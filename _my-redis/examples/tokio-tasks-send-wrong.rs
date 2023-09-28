use std::rc::Rc;
use tokio::task::yield_now;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        tokio::spawn(async {
            let rc = Rc::new("hello");

            // `rc` is used after `.await`. It must be persisted to
            // the task's state.
            yield_now().await;

            println!("{}", rc);
        });
    });
}
