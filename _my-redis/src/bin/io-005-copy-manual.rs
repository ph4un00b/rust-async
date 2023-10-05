use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut file_from = File::open("./resources/read.txt").await?;
    let mut buf = vec![0; 3];
    //?copy the whole file
    loop {
        match file_from.read(&mut buf).await {
            //? Return value of `Ok(0)` signifies that the remote has
            //? closed
            Ok(0) => break,
            Ok(n) => {
                println!("{n} bytes were read!");
                // * Copy the data back to socket
                let mut file_to = File::create("resources/foo-match.txt").await?;
                if file_to.write_all(&buf[..n]).await.is_err() {
                    //? Unexpected socket error. There isn't much we can
                    //? do here so just stop processing.
                    break;
                }
            }
            Err(_) => {
                //? Unexpected socket error. There isn't much we can do
                //? here so just stop processing.
                break;
            }
        }
    }

    Ok::<_, io::Error>(())
}
