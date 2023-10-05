use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> io::Result<()> {
    //? API changed to use stream❗
    let socket = TcpStream::connect("127.0.0.1:6142").await?;
    // * io::split uses an Arc and a Mutex
    let (mut reader_handle, mut writer_handle) = io::split(socket);
    // * To work around this problem, we must split the socket
    // * into a reader handle and a writer handle.
    //? Write data in the background
    tokio::spawn(async move {
        writer_handle.write_all(b"hello\r\n").await?;
        writer_handle.write_all(b"world\r\n").await?;
        //? Sometimes, the rust type inferencer needs
        //? a little help
        Ok::<_, io::Error>(())
    });
    let mut buffer = vec![0; 128];

    loop {
        let n_bytes_read = reader_handle.read(&mut buffer).await?;
        if n_bytes_read == 0 {
            break;
        }
        println!("GOT {:?}", &buffer[..n_bytes_read]);
    }

    Ok(())
}
