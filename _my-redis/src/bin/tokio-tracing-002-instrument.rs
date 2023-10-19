#[tokio::main]
async fn main() -> mini_redis::Result<()> {
    // Open a connection to the mini-redis address.
    let mut client = mini_redis::client::connect("127.0.0.1:6379").await?;

    // Get key "hello"
    let result = client.get("hello").await?;

    println!("jamon!; result={:?}", result);

    Ok(())
}
