#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let _ = tracing_subscriber::fmt::init();

    println!("Hello, world!!");
}
