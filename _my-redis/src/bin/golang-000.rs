use std::io::Write;
use std::thread;
use std::{net::TcpListener, result};

type Result<T> = result::Result<T, ()>;
/*
 * nc "127.0.0.1" 4567
 */

const SAFE_MODE: bool = true;

struct Sensitive<T> {
    inner: T,
}

struct TupleSensitive<T>(T);

//? impl<T> Sensitive<T> {
//?     fn new(inner: T) -> Self {
//?         Self { inner }
//?     }
//? }

impl<T: std::fmt::Display> std::fmt::Display for TupleSensitive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(inner) = self;
        if SAFE_MODE {
            writeln!(f, "[REDACTED]")
        } else {
            inner.fmt(f)
        }
    }
}

fn main() -> Result<()> {
    let add = "127.0.0.1:4567";
    let listener = TcpListener::bind(add).map_err(|err| {
        eprintln!("err: could not bnd {add}, {err}");
    })?;

    println!("listen to {add}");
    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(|| server(rx));

    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                thread::spawn(|| client(s));
                let _ = writeln!(s, "hola!").map_err(|err| eprintln!("cant write: {err}"));
            }
            Err(e) => eprintln!("err: {e}"),
        }
    }
    Ok(())
}

enum Message {
    Connected,
}
fn server(rx: std::sync::mpsc::Receiver<Message>) -> Result<()> {
    todo!()
}

fn client(mut s: std::net::TcpStream, messages: std::sync::mpsc::Sender<Message>) -> Result<()> {
    let _ = writeln!(s, "").map_err(|err| eprintln!("some e: {err}"));
    loop {
        
    }
    todo!()
}
