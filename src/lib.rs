pub mod server;

pub mod client;

pub type Err = Box<dyn std::error::Error + Send + Sync>;

pub type Res<T> = Result<T, Err>;
