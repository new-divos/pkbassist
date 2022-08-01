use clap::Parser;

use nta::{application::Application, cli::Arguments, config::Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Arguments::parse();

    let config = Config::new().await?;
    let app = Application::new(config);

    app.run(&args).await?;

    Ok(())
}
