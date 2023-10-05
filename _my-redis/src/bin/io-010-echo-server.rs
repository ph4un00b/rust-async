use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

fn main() {
    //? multi thread❗
    // let rt = tokio::runtime::Runtime::new().unwrap();
    //? single thread❗
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();

    rt.block_on(async move {
        let listener = match TcpListener::bind("127.0.0.1:6142").await {
            Ok(a) => a,
            Err(_) => return,
        };

        loop {
            println!("loop");
            let (mut socket, _) = match listener.accept().await {
                Ok(a) => a,
                Err(_) => return,
            };
            tokio::spawn(async move {
                println!("spawned");
                echo_task(&mut socket).await;
            });
        }
    });
}

async fn echo_task(stream: &mut TcpStream) {
    let mut buf = vec![0; 1024];

    // * Forgetting to break from the read loop usually results
    // * in a 100% CPU infinite loop situation❗
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => return,
            Ok(n) => {
                println!("{n} bytes were read, {:?}", &buf[..n]);
                if stream.write_all(&buf[..n]).await.is_err() {
                    return;
                }
            }
            Err(_) => {
                return;
            }
        }
    }
}
