use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut file_from = File::open("./resources/read.txt").await?;
    let mut reader = Vec::new();
    //?read the whole file
    file_from.read_to_end(&mut reader).await?;

    let mut file_to = File::create("resources/foo-copy.txt").await?;
    io::copy(&mut &reader[..], &mut file_to).await?;
    Ok(())
}
