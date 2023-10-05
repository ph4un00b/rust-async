use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut file = File::create("./resources/foo-write-all.txt").await?;

    file.write_all(b"all bytes").await?;
    Ok(())
}
