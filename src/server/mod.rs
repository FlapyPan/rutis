use log::{debug, error, info};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc};
use tokio::{time, signal};
use crate::connection::Connection;

#[derive(Debug)]
pub struct Server {
    /// tcp 监听器
    listener: TcpListener,
    /// 关闭时通过广播通知所有连接，由 `run` 的调用方发送广播
    shutdown_sender: broadcast::Sender<()>,
    /// 当所有连接关闭完成时发生广播，服务器可安全退出
    shutdown_complete_sender: mpsc::Sender<()>,
    shutdown_complete_receiver: mpsc::Receiver<()>,
}

impl Server {
    async fn handle(&mut self) -> crate::Res<()> {
        info!("处理传入请求");
        loop {
            let stream = self.accept().await?;
            // 创建一个处理器处理
            let mut handler = ServerHandler {
                connection: Connection::new(stream),
                is_shutdown: false,
                shutdown_receiver: self.shutdown_sender.subscribe(),
                _shutdown_complete_sender: self.shutdown_complete_sender.clone(),
            };
            // 创建一个异步任务处理
            tokio::spawn(async move {
                if let Err(err) = handler.handle().await {
                    error!("连接处理失败 {}", err);
                }
            });
        }
    }

    /// 接收处理连接请求
    ///
    /// 出现错误重试，当重试次数超过 5 次，则失败
    async fn accept(&mut self) -> crate::Res<TcpStream> {
        let mut retry_times = 0;
        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    return Ok(stream);
                }
                Err(err) => {
                    if retry_times > 5 {
                        return Err(err.into());
                    }
                }
            }
            // 等待 10 秒
            time::sleep(time::Duration::from_secs(10)).await;
            retry_times += 1;
        }
    }
}

#[derive(Debug)]
struct ServerHandler {
    connection: Connection,
    /// 是否停机
    is_shutdown: bool,
    /// 接收停机信号
    shutdown_receiver: broadcast::Receiver<()>,
    /// 发送停机完成信号
    _shutdown_complete_sender: mpsc::Sender<()>,
}

impl ServerHandler {
    async fn handle(&mut self) -> crate::Res<()> {
        loop {
            // 如果接收到停机信号退出循环
            if self.is_shutdown { break; }
            // 通过 tokio 的 `select!` 宏来异步处理每次连接
            let frame = tokio::select! {
                // 监听接收 redis 协议帧
                result = self.connection.read() => result?,
                // 接收到停机信号就退出
                _ = self.shutdown_receiver.recv() => {
                    return Ok(());
                }
            };
            let frame = match frame {
                None => { return Ok(()); }
                Some(f) => f,
            };
            debug!("接收到 frame {:?}", frame);
            // todo 解析并执行指令
        }
        Ok(())
    }

    /// 接收处理停机信号
    pub(crate) async fn receive(&mut self) {
        if self.is_shutdown { return; }
        // 等待接收信号
        let _ = self.shutdown_receiver.recv().await;
        self.is_shutdown = true;
    }
}

pub async fn run_server(listener: TcpListener) {
    let (shutdown_sender, _) = broadcast::channel(1);
    let (shutdown_complete_sender, mut shutdown_complete_receiver) = mpsc::channel(1);
    let mut server = Server {
        listener,
        shutdown_sender,
        shutdown_complete_sender,
        shutdown_complete_receiver,
    };
    // ctrl c 信号
    let shutdown_signal = signal::ctrl_c();
    // 通过 tokio 的 `select!` 宏来异步处理每次连接
    tokio::select! {
            result = server.handle() => {
                if let Err(err) = result {
                    error!("处理传入请求失败 {}", err);
                }
            }
            // 当接收到 `shutdown` 关闭
            _ = shutdown_signal => {
                info!("关闭服务器");
            }
        }
    // 关闭广播频道
    drop(server.shutdown_sender);
    drop(server.shutdown_complete_sender);
    // 等待完全退出
    server.shutdown_complete_receiver.recv().await;
}
