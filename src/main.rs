use anyhow::Result;
#[cfg(feature = "gui")]
mod gui;
use pipe::run_cli;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    // Check if user wants CLI mode explicitly
    if args.len() > 1 && (args[1] == "cli" || args[1] == "--cli") {
        return run_cli().await;
    }
    
    // Check for other CLI commands
    if args.len() > 1 && !args[1].starts_with('-') && args[1] != "gui" && args[1] != "--gui" {
        // If it's a valid CLI command, run CLI mode
        return run_cli().await;
    }
    
    // Default to GUI mode
    #[cfg(feature = "gui")]
    {
        gui::run_gui();
        return Ok(());
    }
    
    #[cfg(not(feature = "gui"))]
    {
        // Fallback to CLI if GUI feature is not enabled
        run_cli().await
    }
}