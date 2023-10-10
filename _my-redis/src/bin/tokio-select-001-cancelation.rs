use tokio::sync::oneshot;

async fn some_operation_computed() -> String {
    //? Compute value here
    let _ = 1 + 1;
    "done".to_string()
}

#[tokio::main]
async fn main() {
    let (mut tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();

    tokio::spawn(async {
        //? Select on the operation and the oneshot's
        //? `closed()` notification.
        tokio::select! {
            val = some_operation_computed() => {
                let _ = tx1.send(val);
            }
            _ = tx1.closed() => {
                println!("cancelled");
                //? `some_operation()` is canceled, the
                //? task completes and `tx1` is dropped.
            }
        }
    });

    tokio::spawn(async {
        let _ = tx2.send("two");
    });

    tokio::select! {
        val = rx1 => {
            println!("rx1 completed first with {:?}", val);
        }
        val = rx2 => {
            println!("rx2 completed first with {:?}", val);
        }
    }
}
