// use polars::prelude::*;
use std::env;
use std::error::Error;
use lable_studio_rs::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().collect();
    let config = Config::build(&mut args)?;
    lable_studio_rs::run(config).await?;
    Ok(())
}
