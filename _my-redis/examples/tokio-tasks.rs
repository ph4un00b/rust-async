#[tokio::main]
async fn main() {
    /*
     * Under the hood, they require only a single allocation and
     * 64 bytes of memory. Applications should feel free to spawn
     * thousands, if not millions of tasks❗
     */
    let join_handle = tokio::spawn(async {
        // Do some async work
        "return value"
    });

    // Do some other work
    //? OK
    //? ERR => This happens when the task either panics,
    //? or if the task is forcefully cancelled by the runtime shutting down.
    let out = join_handle.await.unwrap();
    println!("GOT {}", out);
}
