use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use shellexpand::tilde;

mod template;
use template::*;

#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Invoke a template
    Use {
        template: String,
        filename: Vec<String>,
    },
    /// List available templates
    List,
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let args = Args::parse();

    let template_dir = std::path::PathBuf::from(tilde("~/.ptt_templates").to_string());
    if !template_dir.exists() {
        return Err(anyhow!(
            "Must have some plaintext templates in ~/.ptt_templates/"
        ));
    }

    match args.command {
        Command::Use { template, filename } => {
            let filename = if filename.is_empty() {
                None
            } else {
                Some(filename.join("-"))
            };
            let template = Template::new(template)?;
            match template.invoke(filename) {
                Ok(filename_out) => {
                    println!("Created: '{}'", filename_out);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        Command::List => list_available_templates(),
    }
}
