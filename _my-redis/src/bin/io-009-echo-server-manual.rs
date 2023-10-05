use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            // * The strategy is to read some data from the socket into a buffer
            // * then write the contents of the buffer back to the socket
            let mut buf = vec![0; 1024];

            loop {
                match socket.read(&mut buf).await {
                    //? Return value of `Ok(0)` signifies that the remote has
                    //? closed
                    // *  Forgetting to break from the read loop on EOF
                    // * is a common source of bugs.❗
                    Ok(0) => return,
                    Ok(n) => {
                        println!("{n} bytes were read, {:?}", &buf[..n]);
                        // * Copy the data back to socket
                        if socket.write_all(&buf[..n]).await.is_err() {
                            //? Unexpected socket error. There isn't much we can
                            //? do here so just stop processing.
                            return;
                        }
                    }
                    Err(_) => {
                        //? Unexpected socket error. There isn't much we can do
                        //? here so just stop processing.
                        return;
                    }
                }
            }
        });
    }
}
