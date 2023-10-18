#[tokio::main]
async fn main() {
    //? ... spawn application as separate task ...
    match tokio::signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
            //? we also shut down in case of error
        }
    }
    //? send shutdown signal to application and wait
}
