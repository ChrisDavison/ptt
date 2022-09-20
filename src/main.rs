#[macro_use]
extern crate lazy_static;

use anyhow::{anyhow, Result};
use clap::{AppSettings, Parser, Subcommand};
use shellexpand::tilde;

mod template;
mod utility;
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
        #[clap(short = 'f', long, default_value = "%Y-%m-%d")]
        /// Date format to use for dated templates
        format: String,
    },
    /// List available templates
    List,
    #[clap(setting=AppSettings::Hidden)]
    /// Display a manpage that can be exported to $HOME/.local/share/man/ptt.1
    Man,
    #[clap(setting=AppSettings::Hidden)]
    Test,
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
        Command::Use {
            template,
            filename,
            format,
        } => {
            let filename = if filename.is_empty() {
                None
            } else {
                Some(filename.join("-"))
            };
            let template = Template::new(template, format)?;
            template.invoke(filename).map(|fn_out| {
                println!("Created '{}'", fn_out);
            })
        }
        Command::List => list_available_templates(),
        Command::Man => todo!(),
        Command::Test => {
            let original = "{{this}} {{is}} {{a}} {{test}}";
            let mut moustaches = template::find_moustaches(original);
            template::get_response_for_moustaches(&mut moustaches)?;
            let _replaced = template::replace_moustaches(original, moustaches);
            Ok(())
        }
    }
}
