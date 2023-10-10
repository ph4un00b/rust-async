use std::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> io::Result<()> {
    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        tx.send(()).unwrap();
    });

    let mut listener = TcpListener::bind("localhost:3465").await?;

    tokio::select! {
        res = async {
            loop {
                let (socket, _) = listener.accept().await?;
                tokio::spawn(async move { process(&socket).await });
            }
            //? Help the rust type inferencer out
            Ok::<_, io::Error>(())
        } => {
            /*
             * Notice listener.accept().await?.
             * The ? operator propagates the error out of that
             * expression and to the res binding.
             */
            res?
        }
        _ = rx => {
            println!("terminating accept loop");
        }
    }

    Ok(())
}

async fn process(socket: &TcpStream) {
    println!("socket {socket:?}")
}
