use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut f = File::open("./resources/foo.txt").await?;
    let mut buffer = Vec::new();

    //?read the whole file
    f.read_to_end(&mut buffer).await?;
    println!("All bytes: {:?}", &buffer[..]);
    Ok(())
}
