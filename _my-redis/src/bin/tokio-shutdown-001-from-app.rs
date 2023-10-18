#[tokio::main]
async fn main() {
    let (shutdown_send, mut shutdown_recv) = tokio::sync::mpsc::unbounded_channel::<()>();
    //? ... spawn application as separate task ...
    //?
    //? application uses shutdown_send in case a shutdown was issued from inside
    //? the application
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = shutdown_recv.recv() => {},
    }
    //? send shutdown signal to application and wait
}
