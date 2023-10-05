use std::io::Cursor;

use bytes::Buf;

// * @see https://redis.io/docs/reference/protocol-spec/
//? pub enum Frame {
//?     Simple(String),
//?     Error(String),
//?     Integer(u64),
//?     Bulk(Bytes),
//?     Array(Vec<Frame>),
//?     Nil,
//? }

use bytes::BytesMut;
use mini_redis::Frame;
use mini_redis::Result;
use tokio::io;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufWriter;
use tokio::net::TcpStream;

pub struct Connection {
    buf_stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            buf_stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4096),
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        let mut cursor_buf = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut cursor_buf) {
            Ok(_) => {
                let frame_size = cursor_buf.position() as usize;
                cursor_buf.set_position(0);
                let frame = Frame::parse(&mut cursor_buf)?;
                self.buffer.advance(frame_size);
                Ok(Some(frame))
            }
            Err(mini_redis::frame::Error::Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    //? Write a decimal frame to the stream
    async fn write_decimal(&mut self, val: u64) -> io::Result<()> {
        use std::io::Write;

        //? Convert the value to a string
        let mut buf = [0u8; 12];
        let mut cursor_buf = Cursor::new(&mut buf[..]);
        write!(&mut cursor_buf, "{}", val)?;

        let current_position = cursor_buf.position() as usize;
        self.buf_stream
            .write_all(&cursor_buf.get_ref()[..current_position])
            .await?;
        self.buf_stream.write_all(b"\r\n").await?;

        Ok(())
    }
}

impl Connection {
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if 0 == self.buf_stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }
        }
    }

    //? Write a frame to the connection.
    /*
     *  In order to minimize write syscalls, writes will be buffered.
     *
     * Syscalls involve transitioning from user mode to kernel mode
     * in the operating system, which incurs a performance overhead.
     *
     * Each syscall operation typically involves a context switch,
     * where the operating system saves the current state of the
     * user process and loads the state of the kernel to perform
     * the requested operation.
     */
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        match frame {
            Frame::Simple(val) => {
                self.buf_stream.write_u8(b'+').await?;
                self.buf_stream.write_all(val.as_bytes()).await?;
                self.buf_stream.write_all(b"\r\n").await?;
            }
            Frame::Error(val) => {
                self.buf_stream.write_u8(b'-').await?;
                self.buf_stream.write_all(val.as_bytes()).await?;
                self.buf_stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(val) => {
                self.buf_stream.write_u8(b':').await?;
                self.write_decimal(*val).await?;
            }
            Frame::Null => {
                self.buf_stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(val) => {
                let len = val.len();

                self.buf_stream.write_u8(b'$').await?;
                self.write_decimal(len as u64).await?;
                self.buf_stream.write_all(val).await?;
                self.buf_stream.write_all(b"\r\n").await?;
            }
            Frame::Array(_val) => unimplemented!(),
        }

        let _ = self.buf_stream.flush().await?;
        /*
         * The function ends with a call to self.stream.flush().await.
         * Because BufWriter stores writes in an intermediate buffer
         *
         * calls to write do not guarantee that the data is written to the socket
         *
         * Before returning, we want the frame to be written to the socket.
         * The call to flush() writes any data pending in the buffer to the socket.
         *
         * Another alternative would be to not call flush() in write_frame().
         * Instead, provide a flush() function on Connection.
         * This would allow the caller to write queue multiple small
         * frames in the write buffer then write them all to the socket
         * with one write syscall.
         *
         * Doing this complicates the Connection API.
         * Simplicity is one of Mini-Redis' goals,
         * so we decided to include the flush().await call in fn write_frame().
         */

        Ok(())
    }
}
fn main() {}
