use tokio::fs::File;
use tokio::io::{self, AsyncReadExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    //? with ./resources/foo.txt or resources/foo.txt is the same in windows
    let mut fs = File::open("resources/foo.txt").await?;
    let mut buffer = [0; 10];

    //? read up to 10 bytes
    let n = fs.read(&mut buffer[..]).await?;

    println!("The bytes: {:?}", &buffer[..n]);
    Ok(())
}
