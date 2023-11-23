use std::io::Cursor;
use tokio::io::{AsyncReadExt, BufWriter};
use tokio::net::TcpStream;
use bytes::BytesMut;
use crate::frame::{Frame, FrameError};

/// 接收和发送 redis 协议帧
#[derive(Debug)]
pub(crate) struct Connection {
    /// 写缓冲区
    writer: BufWriter<TcpStream>,
    /// 读缓冲区
    buf: BytesMut,
}

impl Connection {
    /// 创建一个新的连接
    pub(crate) fn new(stream: TcpStream) -> Connection {
        Connection {
            writer: BufWriter::new(stream),
            // 4kb 大小的缓冲区
            buf: BytesMut::with_capacity(4096),
        }
    }

    /// 读取一个 RESP 帧
    pub(crate) async fn read(&mut self) -> crate::Res<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }
            if self.writer.read_buf(&mut self.buf).await? == 0 {
                return if self.buf.is_empty() {
                    Ok(None)
                } else {
                    Err("连接被重置".into())
                }
            }
        }
    }

    /// 转换获取 RESP 帧
    fn parse_frame(&mut self) -> crate::Res<Option<Frame>> {
        let mut cursor = Cursor::new(&self.buf[..]);
        match Frame::parse(&mut cursor) {
            Ok(frame) => Ok(Some(frame)),
            Err(FrameError::Incomplete) => Ok(None),
            Err(err) => Err(err.into())
        }
    }

    pub(crate) async fn write(&mut self) -> ! {
        todo!("发送 redis 协议帧")
    }
}
