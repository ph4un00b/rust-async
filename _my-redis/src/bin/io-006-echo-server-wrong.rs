use tokio::io;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6142").await?;

    loop {
        //? This single value implements both AsyncRead and AsyncWrite
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            // Copy data here
            //? This fails to compile
            io::copy(&mut socket, &mut socket).await
        });
    }
}
