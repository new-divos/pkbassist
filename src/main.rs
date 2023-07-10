use clap::Parser;

use pkbassist::{application::Application, cli::Arguments, config::Config, error::Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Arguments::parse();
    let config = Config::new().await?;

    Application::setup_logger(&args, &config)?;

    let config = config.load().await?;
    let app = Application::new(config);

    app.run(&args).await
}
