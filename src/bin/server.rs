use clap::Parser;
use tokio::net::TcpListener;
use rutis::server::run_server;

/// 服务器参数
#[derive(Parser, Debug)]
#[command(name = "rutis-server", version, author)]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,
    #[arg(long, default_value = "6379")]
    port: u16,
}

/// rutis 服务器主入口，使用 `clap` 处理命令行参数
#[tokio::main]
pub async fn main() -> rutis::Res<()> {
    let args = Args::parse();
    let addr = format!("{}:{}", args.bind, args.port);
    let listener = TcpListener::bind(&addr).await?;
    run_server(listener).await;
    Ok(())
}
