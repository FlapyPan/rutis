//! 处理 RESP 协议帧

use std::fmt;
use std::io::Cursor;
use std::num::TryFromIntError;
use std::string::FromUtf8Error;
use bytes::{Buf, Bytes};

/// RESP 数据类型
#[derive(Debug, Clone)]
pub(crate) enum Frame {
    /// 单行字符串
    SimpleString(String),
    /// 错误
    Error(String),
    /// 整型
    Integer(i64),
    /// 多行字符串
    BulkString(Bytes),
    // 数组
    Array(Vec<Frame>),
    // 不存在的值
    Null(),
}

impl Frame {
    pub(crate) fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Frame, FrameError> {
        match get_u8(cursor)? {
            b'+' => {
                let line = get_line(cursor)?.to_vec();
                let str = String::from_utf8(line)?;
                Ok(Frame::SimpleString(str))
            }
            _ => unimplemented!(),
        }
    }
}

/// 获取整个字节数组
fn get_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !cursor.has_remaining() { return Err(FrameError::Incomplete); }
    Ok(cursor.get_u8())
}

/// 获取一行字符串
fn get_line<'a>(cursor: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], FrameError> {
    let cursor_ref = cursor.get_ref();
    let start = cursor.position() as usize;
    let end = cursor_ref.len() - 1;
    // 循环查找第一个 \r\n
    for i in start..end {
        if cursor_ref[i] == b'\r' && cursor_ref[i + 1] == b'\n' {
            cursor.set_position((i + 2) as u64);
            return Ok(&cursor.get_ref()[start..i]);
        }
    }
    Err(FrameError::Incomplete)
}

/// RESP 错误类型
#[derive(Debug)]
pub(crate) enum FrameError {
    /// 不完整的数据
    Incomplete,
    /// 其他错误
    Other(crate::Err),
}

impl std::error::Error for FrameError {}

impl From<String> for FrameError {
    fn from(value: String) -> Self {
        FrameError::Other(value.into())
    }
}

impl From<&str> for FrameError {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<FromUtf8Error> for FrameError {
    fn from(_value: FromUtf8Error) -> Self {
        "错误的协议格式".into()
    }
}

impl From<TryFromIntError> for FrameError {
    fn from(_value: TryFromIntError) -> Self {
        "错误的协议格式".into()
    }
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrameError::Incomplete => "不完整的信息".fmt(f),
            FrameError::Other(err) => err.fmt(f),
        }
    }
}
