#[tokio::main]
async fn main() {
    let token = tokio_util::sync::CancellationToken::new();
    let cloned_token = token.clone();

    let join_handle = tokio::spawn(async move {
        //? Wait for either cancellation or a very long time
        tokio::select! {
            _ = cloned_token.cancelled() => {
                println!("will cancel from token");
                //? The token was cancelled
                5
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(9999)) => {
                println!("will cancel from this sleep");
                99
            }
        }
    });

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        println!("sleep 1");
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        println!("sleep 2");
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        println!("sleep 8");
        tokio::time::sleep(std::time::Duration::from_millis(9999)).await;
        println!("cancel");
        token.cancel();
    });

    println!("result {}", join_handle.await.unwrap());
}
