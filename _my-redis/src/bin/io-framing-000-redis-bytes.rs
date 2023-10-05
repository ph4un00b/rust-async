use bytes::Bytes;

pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Array(Vec<Frame>),
    Nil,
}

use bytes::BytesMut;
use mini_redis::Result;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream,
            //? Allocate the buffer with 4kb of capacity.
            buffer: BytesMut::with_capacity(4096),
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
        /*
         * Because pairing a byte array with a cursor is very common,
         * the bytes crate provides an abstraction representing a
         * byte array and cursor.
         */
        loop {
            //? On success, the number of bytes is returned. `0`
            //? indicates "end of stream".
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                //? The remote closed the connection. For this to be
                //? a clean shutdown, there should be no data in the
                //? read buffer. If there is, this means that the
                //? peer closed the socket while sending a frame.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
            //? Attempt to parse a frame from the buffered data. If
            //? enough data has been buffered, the frame is
            //? returned.
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
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
