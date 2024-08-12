use lable_studio_rs::Config;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().collect();
    let config = Config::build(&mut args)?;
    lable_studio_rs::run(config).await?;
    Ok(())
}
