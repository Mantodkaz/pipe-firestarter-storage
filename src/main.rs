use anyhow::Result;
#[cfg(feature = "gui")]
mod gui;
use pipe::run_cli;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "gui" || args[1] == "--gui") {
        #[cfg(feature = "gui")]
        {
            gui::run_gui();
            return Ok(());
        }
    }
    run_cli().await
}