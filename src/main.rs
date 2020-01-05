use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use shellexpand::tilde;

type Result<T> = std::result::Result<T, Box<dyn ::std::error::Error>>;

static USAGE: &'static str = "usage: ptt [-l|-h] <template> [<filename>]";

static LONG_USAGE: &'static str = r#"ptt: plaintext template tool

usage:
    ptt [-l|-h] <template> [<filename>]

This tool looks for templates in ~/.ptt_templates, and copies them in to the 
current directory, under a given filename.

Auto-dating
    If the templatename starts with 'DATE-', today's date in format
    YYYYMMDD will be used in the filename. The filename is ONLY optional if 
    the template has the 'DATE-' prefix. When showing the template list,
    the 'DATE-' prefix is removed and shown as (D) instead, and 'DATE-' doesn't
    need to be specified when invoking ptt (it'll automatically try to find
    DATE-<template>, and if it fails will then look for <template>).
"#;

struct Template {
    name: String,
    filepath: PathBuf,
    dated: bool,
}

impl Template {
    fn from_arg(arg: Option<impl ToString>) -> Result<Template> {
        let templatename = match arg {
            None => return Err(USAGE.into()),
            Some(t) => t.to_string(),
        };
        let fn_with_date = PathBuf::from(
            tilde(&format!("~/.ptt_templates/DATE-{}.txt", templatename)).to_string(),
        );
        let fn_no_date =
            PathBuf::from(tilde(&format!("~/.ptt_templates/{}.txt", templatename)).to_string());
        if fn_with_date.exists() {
            Ok(Template {
                name: templatename,
                filepath: fn_with_date,
                dated: true,
            })
        } else if fn_no_date.exists() {
            Ok(Template {
                name: templatename,
                filepath: fn_no_date,
                dated: false,
            })
        } else {
            Err(LONG_USAGE.into())
        }
    }

    fn invoke(&self, filename: Option<Vec<impl ToString>>) -> Result<String> {
        let now: DateTime<Utc> = Utc::now();
        let nowstr = now.format("%Y%m%d").to_string();

        // dated & filename = date-filename
        // dated & no filename = date-templatename
        // no date & filename = filename
        // no date & no filename = return err
        let fname = match filename {
            Some(f) => {
                let mut out = String::new();
                for s in f {
                    out.push_str(s.to_string().as_ref())
                }
                out
            }
            None => String::new(),
        };
        if self.dated && !fname.is_empty() {
            let fname = vec![nowstr, fname].join("-") + ".txt";
            fs::copy(self.filepath.clone(), fname.clone())
                .expect("Failed to copy template to file");
            Ok(fname)
        } else if self.dated {
            let fname = vec![nowstr, self.name.clone()].join("-") + ".txt";
            fs::copy(self.filepath.clone(), fname.clone())
                .expect("Failed to copy template to file");
            Ok(fname)
        } else if !fname.is_empty() {
            fs::copy(self.filepath.clone(), fname.clone() + ".txt")
                .expect("Failed to copy template to file");
            Ok(fname)
        } else {
            Err(LONG_USAGE.into())
        }
    }
}

fn get_template_name(p: PathBuf) -> String {
    let fname = p.file_name().unwrap().to_string_lossy().to_string();
    let no_ext_fname = fname.trim_end_matches(".txt");
    if no_ext_fname.starts_with("DATE-") {
        let no_ext_no_date_fname = no_ext_fname.trim_start_matches("DATE-");
        format!("{} (D)", no_ext_no_date_fname.to_string())
    } else {
        no_ext_fname.to_string()
    }
}

fn list_available_templates() -> Result<()> {
    let mut templates = Vec::new();
    let template_dir = PathBuf::from(tilde("~/.ptt_templates").to_string());
    for entry in template_dir.read_dir()? {
        if let Ok(entry) = entry {
            if entry.path().is_file() {
                templates.push(get_template_name(entry.path()));
            }
        }
    }
    println!("Templates: {}", templates.join(", "));

    Ok(())
}

fn main() {
    let template_dir = std::path::PathBuf::from(tilde("~/.ptt_templates").to_string());
    if !template_dir.exists() {
        eprintln!("Must have some plaintext templates in ~/.ptt_templates/");
        return;
    }

    let all_args: Vec<String> = std::env::args().skip(1).collect();
    let flags: Vec<String> = all_args
        .iter()
        .filter(|&x| x.starts_with("-"))
        .map(|x| x.to_owned())
        .collect();
    let args: Vec<String> = all_args
        .iter()
        .filter(|&x| !x.starts_with("-"))
        .map(|x| x.to_owned())
        .collect();

    if flags.contains(&"-h".to_string()) {
        println!("{}", USAGE);
    } else if flags.contains(&"--help".to_string()) {
        println!("{}", LONG_USAGE);
    } else if flags.contains(&"-l".to_string()) || args.is_empty() {
        list_available_templates().unwrap();
    } else {
        let template = match Template::from_arg(args.get(0).map(|x| x.to_owned())) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        };
        match template.invoke(args.get(1..).map(|x| x.to_owned())) {
            Ok(filename_out) => println!("Created: '{}'", filename_out),
            Err(e) => eprintln!("{}", e),
        }
    }
}
