use chrono::Utc;
use shellexpand::tilde;
use std::io::Write;

type Result<T> = std::result::Result<T, Box<dyn ::std::error::Error>>;

use hmac_sha256::Hash;

static USAGE: &'static str = "usage: ptt [-l] <template> <filename>";

fn prompt_for_string(prompt: &str) -> String {
    let mut response = String::new();
    print!("{}: ", prompt);
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut response).unwrap();
    format!("{}: {}\n", prompt, response.trim())
}

fn prompt_for_hash() -> String {
    let mut response = String::new();
    print!("to hash: ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut response).unwrap();
    let hashed = Hash::hash(&response.trim().as_bytes());
    format!("hash: {}\n", String::from_utf8_lossy(&hashed))
}

fn prompt_for_tags() -> String {
    let mut response = String::new();
    print!("tags: ");
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut response).unwrap();
    let tags: Vec<String> = response
        .trim()
        .split(" ")
        .map(|x| format!("@{}", x))
        .collect();
    format!("{}\n", tags.join(" "))
}

fn get_matching_template(templatename: &str) -> Result<String> {
    let mut template = std::path::PathBuf::from(tilde("~/.ptt_templates").to_string());
    template.push(format!("{}.txt", templatename));
    let contents = std::fs::read_to_string(template)?;
    Ok(contents)
}

fn list_available_templates() -> Result<()> {
    let mut templates = Vec::new();
    let template_dir = std::path::PathBuf::from(tilde("~/.ptt_templates").to_string());
    for entry in template_dir.read_dir()? {
        if let Ok(entry) = entry {
            let fname = entry.file_name();
            templates.push(fname.to_string_lossy().to_string());
        }
    }
    println!("Available templates");
    println!("{}", templates.join(", "));

    Ok(())
}

fn main() -> Result<()> {
    get_matching_template("bookmark")?;
    let template_dir = std::path::PathBuf::from(tilde("~/.ptt_templates").to_string());
    if !template_dir.exists() {
        return Err(format!("Must have some plaintext templates in ~/.ptt_templates/").into());
    }

    let mut args = Vec::new();
    for arg in std::env::args().skip(1) {
        if arg == "-l" {
            return list_available_templates();
        } else {
            args.push(arg);
        }
    }
    let (template, filename) = if args.len() < 1 {
        return Err(format!("{}", USAGE).into());
    } else if args.len() == 1 {
        let mut response = String::new();
        print!("Template: ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut response).unwrap();
        (response, args[0].clone())
    } else {
        (
            args[0].clone(),
            format!("{}.txt", args[1..].join("-").clone()),
        )
    };

    println!("Filling template: `{}` for file `{}`\n", template, filename);

    let mut filled_template = String::new();
    for line in get_matching_template(&template)?.lines() {
        let parts: Vec<&str> = line.split(": ").collect();
        let prompt = parts[0];
        let replace = parts[1..].join(": ");
        if line.is_empty() {
            filled_template += "\n";
        } else if replace == "<DATE>" {
            let date = Utc::now().format("%Y-%m-%dT%H:%M:%S%Z");
            filled_template += &format!("date: {}\n", date).to_string();
        } else if replace == "<HASH>" {
            filled_template += &prompt_for_hash();
        } else if line == "<TAGS>" {
            filled_template += &prompt_for_tags();
        } else {
            filled_template += &prompt_for_string(prompt);
        };
    }
    println!("{}", filled_template);
    std::fs::write(filename, filled_template)?;
    Ok(())
}
