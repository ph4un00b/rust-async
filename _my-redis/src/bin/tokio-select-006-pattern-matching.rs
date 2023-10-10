use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx1, mut rx1) = mpsc::channel(128);
    let (tx2, mut rx2) = mpsc::channel(128);

    tokio::spawn(async move {
        let _ = tx1.send(Some("one"));
        let _ = tx2.send("two").await;
    });

    tokio::select! {
        /*
         * In this example, the select! expression waits on
         * receiving a value from rx1 and rx2. If a channel closes,
         * recv() returns None.
         * This does not match the pattern and the branch is disabled.
         * The select! expression will continue waiting on the remaining branches.
         */
        Some(v) = rx1.recv() => {
            println!("Got {:?} from rx1", v);
        }
        Some(v) = rx2.recv() => {
            println!("Got {:?} from rx2", v);
        }
        else => {
            println!("Both channels closed");
        }
    }
}
