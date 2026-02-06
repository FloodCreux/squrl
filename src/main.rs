extern crate core;

mod app;
mod models;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");

    Ok(())
}
