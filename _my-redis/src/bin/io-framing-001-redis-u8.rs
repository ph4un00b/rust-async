use bytes::Bytes;

pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Array(Vec<Frame>),
    Nil,
}

use mini_redis::Result;
use tokio::{io::AsyncReadExt, net::TcpStream};

//? without BytesMut
pub struct Connection {
    stream: TcpStream,
    buffer: Vec<u8>,
    cursor: usize,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            //? Allocate the buffer with 4kb of capacity.
            buffer: vec![0; 4096],
            cursor: 0,
        }
    }

    fn parse_frame(&self) -> Result<Option<Frame>> {
        todo!()
    }
}

impl Connection {
    //? Read a frame from the connection.
    //?
    //? Returns `None` if EOF is reached
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            //? Ensure the buffer has capacity
            if self.buffer.len() == self.cursor {
                //? Grow the buffer
                /*
                 * when using Vec<u8>, the buffer must be initialized.
                 * vec![0; 4096] allocates an array of 4096 bytes and writes
                 * zero to every entry. When resizing the buffer,
                 * the new capacity must also be initialized with zeros.
                 * The initialization process is not free.
                 * When working with BytesMut and BufMut, capacity is uninitialized.
                 * The BytesMut abstraction prevents us from reading the
                 * uninitialized memory. This lets us avoid the initialization step.
                 */
                self.buffer.resize(self.cursor * 2, 0);
            }

            //? Read into the buffer, tracking the number
            //? of bytes read
            let n = self.stream.read(&mut self.buffer[self.cursor..]).await?;

            if 0 == n {
                if self.cursor == 0 {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            } else {
                //? Update our cursor
                self.cursor += n;
            }
        }
    }

    //? Write a frame to the connection.
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        // implementation here
        todo!()
    }
}
fn main() {}
