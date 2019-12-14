use shellexpand::tilde;
use std::fs;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Box<dyn ::std::error::Error>>;

static USAGE: &'static str = "usage: ptt [-l] <template> <filename>";

fn list_available_templates() -> Result<()> {
    let mut templates = Vec::new();
    let template_dir = PathBuf::from(tilde("~/.ptt_templates").to_string());
    for entry in template_dir.read_dir()? {
        if let Ok(entry) = entry {
            if entry.path().is_file() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.ends_with(".txt") {
                    templates.push(fname[..fname.len() - 4].to_string());
                } else {
                    templates.push(fname);
                }
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

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().filter(|&x| x == "-l").count() > 0 {
        list_available_templates().unwrap();
        return;
    }

    let templatename = match args.get(0) {
        Some(t) => t,
        None => {
            println!("Not enough args: {}", USAGE);
            return;
        }
    };
    let filename = match args.get(1..) {
        Some(f) => f.join("-") + ".txt",
        None => {
            println!("Not enough args: {}", USAGE);
            return;
        }
    };

    let templatefn =
        PathBuf::from(tilde(&format!("~/.ptt_templates/{}.txt", templatename)).to_string());
    if templatefn.exists() {
        fs::copy(templatefn, filename).expect("Failed to copy template to file");
    } else {
        println!("Template `{}` doesn't exist", templatename);
    }
}
