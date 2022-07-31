use clap::Parser;

use nta::{application::Application, cli::Arguments, config::Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = Config::new().await?;
    let app = Application::new(config);

    let args = Arguments::parse();
    app.run(&args).await?;

    Ok(())
}
