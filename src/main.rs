use clap::Parser;

use notes_attendant::{
    application::Application,
    config::{Arguments, Config},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = Config::new()?;
    let app = Application::new(config);

    let args = Arguments::parse();
    app.run(&args).await?;

    Ok(())
}
