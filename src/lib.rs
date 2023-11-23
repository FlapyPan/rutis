pub mod server;
pub mod client;
pub(crate) mod connection;
pub(crate) mod frame;

pub type Err = Box<dyn std::error::Error + Send + Sync>;

pub type Res<T> = Result<T, Err>;
