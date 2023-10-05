use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut file = File::create("resources/foo.txt").await?;

    //? Writes some prefix of the byte string, but not necessarily all of it.
    let n_bytes_written = file.write(b"some bytes").await?;

    println!("Wrote the first {} bytes of 'some bytes'.", n_bytes_written);
    Ok(())
}
