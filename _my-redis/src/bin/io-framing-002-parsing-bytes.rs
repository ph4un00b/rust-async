use std::io::Cursor;

use bytes::Buf;

// pub enum Frame {
//     Simple(String),
//     Error(String),
//     Integer(u64),
//     Bulk(Bytes),
//     Array(Vec<Frame>),
//     Nil,
// }

use bytes::BytesMut;
use mini_redis::Frame;
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
            buffer: BytesMut::with_capacity(4096),
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        //? Create the `T: Buf` type.
        let mut buf = Cursor::new(&self.buffer[..]);
        //? Check whether a full frame is available
        //? @see https://github.com/tokio-rs/mini-redis/blob/tutorial/src/frame.rs#L65-L103
        //? @see api https://docs.rs/bytes/1.5.0/bytes/buf/trait.Buf.html
        match Frame::check(&mut buf) {
            Ok(_) => {
                //? Get the byte length of the frame
                let frame_len = buf.position() as usize;
                //? Reset the internal cursor for the
                //? call to `parse`.
                buf.set_position(0);
                //? Parse the frame
                let frame = Frame::parse(&mut buf)?;
                //? Discard the frame from the buffer
                self.buffer.advance(frame_len);
                //? Return the frame to the caller.
                Ok(Some(frame))
            }
            //? Not enough data has been buffered❗
            Err(mini_redis::frame::Error::Incomplete) => Ok(None),
            //? An error was encountered
            Err(e) => Err(e.into()),
        }
    }
}

impl Connection {
    //? Read a frame from the connection.
    //?
    //? Returns `None` if EOF is reached
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
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
