/// rutis 服务器主入口
#[tokio::main]
pub async fn main() -> rutis::Res<()> {
    println!("hello rutis server!");
    Ok(())
}
