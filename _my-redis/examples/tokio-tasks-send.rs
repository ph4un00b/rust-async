use std::rc::Rc;
use tokio::task::yield_now;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        tokio::spawn(async {
            // The scope forces `rc` to drop before `.await`.
            {
                let rc = Rc::new("\nhello");
                println!("{}", rc);
            }

            // `rc` is no longer used. It is **not** persisted when
            // the task yields to the scheduler
            yield_now().await;
        });
    });
}
