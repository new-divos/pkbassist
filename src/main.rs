use clap::Parser;

use nta::{
    application::Application,
    cli::Arguments,
    config::{Config, Options},
    error::Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Arguments::parse();
    let options = Options::new().await?;

    Application::setup_logger(&args, &options)?;

    let config = Config::new_old(&options).await?;
    let app = Application::new(config);

    app.run(&args).await
}
