use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use shellexpand::tilde;
use std::fs;
use std::path::PathBuf;

pub struct Template {
    name: String,
    filepath: PathBuf,
    dated: bool,
}

impl Template {
    pub fn new(template_name: impl ToString) -> Result<Template> {
        let template_name = template_name.to_string();
        let fn_with_date = PathBuf::from(
            tilde(&format!("~/.ptt_templates/DATE-{}.txt", template_name)).to_string(),
        );
        let fn_no_date =
            PathBuf::from(tilde(&format!("~/.ptt_templates/{}.txt", template_name)).to_string());
        if fn_with_date.exists() {
            Ok(Template {
                name: template_name,
                filepath: fn_with_date,
                dated: true,
            })
        } else if fn_no_date.exists() {
            Ok(Template {
                name: template_name,
                filepath: fn_no_date,
                dated: false,
            })
        } else {
            Err(anyhow!("BLAH"))
        }
    }

    pub fn invoke(&self, filename: Option<impl ToString>) -> Result<String> {
        let now: DateTime<Utc> = Utc::now();
        let nowstr = now.format("%Y%m%d").to_string();

        // dated & filename = date-filename
        // dated & no filename = date-templatename
        // no date & filename = filename
        // no date & no filename = return err
        let fname = match filename {
            Some(f) => f.to_string(),
            None => String::new(),
        };
        match (self.dated, fname.is_empty()) {
            (true, false) => {
                let fname = vec![nowstr, fname].join("-") + ".txt";
                fs::copy(self.filepath.clone(), fname.clone())
                    .expect("Failed to copy template to file");
                Ok(fname)
            }
            (true, true) => {
                let fname = vec![nowstr, self.name.clone()].join("-") + ".txt";
                fs::copy(self.filepath.clone(), fname.clone())
                    .expect("Failed to copy template to file");
                Ok(fname)
            }
            (false, true) => {
                fs::copy(self.filepath.clone(), fname.clone() + ".txt")
                    .expect("Failed to copy template to file");
                Ok(fname)
            }
            (false, false) => Err(anyhow!("Filename empty and not dated")),
        }
    }
}

pub fn get_template_name(p: PathBuf) -> String {
    let fname = p.file_name().unwrap().to_string_lossy().to_string();
    let no_ext_fname = fname.trim_end_matches(".txt");
    if no_ext_fname.starts_with("DATE-") {
        let no_ext_no_date_fname = no_ext_fname.trim_start_matches("DATE-");
        format!("(DATE) {}", no_ext_no_date_fname)
    } else {
        no_ext_fname.to_string()
    }
}

pub fn list_available_templates() -> Result<()> {
    let mut templates = Vec::new();
    let template_dir = PathBuf::from(tilde("~/.ptt_templates").to_string());
    for entry in (template_dir.read_dir()?).flatten() {
        if entry.path().is_file() {
            templates.push(get_template_name(entry.path()));
        }
    }
    println!("Templates\n");
    templates.sort();
    for template in templates {
        println!("- {}", template);
    }

    Ok(())
}
