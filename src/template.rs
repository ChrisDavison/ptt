use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use regex::Regex;
use shellexpand::tilde;
use std::fs;
use std::path::PathBuf;

lazy_static! {
    static ref RE: Regex = Regex::new("\\{\\{(?P<moustache>[a-zA-Z]+)\\}\\}")
        .expect("Couldn't create moustache regex");
}

pub struct Template {
    name: String,
    filepath: PathBuf,
    dated: bool,
    dateformat: String,
    verbose: bool,
}

impl Template {
    pub fn new(template_name: impl ToString, format: String, verbose: bool) -> Result<Template> {
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
                dateformat: format,
                verbose,
            })
        } else if fn_no_date.exists() {
            Ok(Template {
                name: template_name,
                filepath: fn_no_date,
                dated: false,
                dateformat: format,
                verbose,
            })
        } else {
            Err(anyhow!(
                "Couldn't find template `{template_name}`, with DATE- or without"
            ))
        }
    }

    pub fn invoke(&self, filename: Option<impl ToString>) -> Result<String> {
        let now: DateTime<Utc> = Utc::now();
        let nowstr = now.format(&self.dateformat).to_string();

        // If we give a filename, use that for the output.
        // If not, use the name of the template for the output
        let fname = match &filename {
            Some(f) => f.to_string(),
            None => self.name.clone(),
        };
        let contents = std::fs::read_to_string(&self.filepath)?;
        if self.verbose {
            println!("{}", contents);
        }
        let new_contents = find_and_replace_moustaches(&contents)?;
        if self.dated {
            let fname = vec![nowstr, self.name.clone()].join("-") + ".txt";
            fs::write(fname.clone(), new_contents)?;
            Ok(fname)
        } else if filename.is_some() {
            fs::write(fname.clone() + ".txt", new_contents)?;
            Ok(fname)
        } else {
            Err(anyhow!("Filename empty and not dated"))
        }
    }
}

fn get_template_name(p: PathBuf) -> String {
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

pub fn find_moustaches(txt: &str) -> IndexMap<String, String> {
    let mut moustaches = IndexMap::new();
    for cap in RE.captures_iter(txt) {
        moustaches.insert(cap["moustache"].into(), String::new());
    }
    moustaches
}

pub fn get_response_for_moustaches(moustaches: &mut IndexMap<String, String>) -> Result<()> {
    for (k, v) in moustaches.iter_mut() {
        *v = crate::utility::read_from_stdin(&format!("{k} ⇒ "))?;
    }
    Ok(())
}

pub fn replace_moustaches(txt: &str, map: IndexMap<String, String>) -> String {
    let mut txt = txt.to_string();
    for (k, v) in map {
        let to_rep = format!("{{{{{k}}}}}");
        txt = txt.replace(&to_rep, &v).to_string();
    }
    txt
}

fn find_and_replace_moustaches(txt: &str) -> Result<String> {
    let mut moustaches = find_moustaches(txt);
    get_response_for_moustaches(&mut moustaches)?;
    Ok(replace_moustaches(txt, moustaches))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_moustaches_in_template_test() {
        let txt = "{{this}} {{is}} {{this}} {{again}}";
        let mut map = IndexMap::new();
        map.insert("this".to_string(), String::new());
        map.insert("is".to_string(), String::new());
        map.insert("again".to_string(), String::new());
        assert_eq!(find_moustaches(txt), map);
    }

    #[test]
    fn replace_in_template_test() {
        let txt = "{{this}} {{is}} {{this}} {{again}}";
        let out = "another boy another day";
        let mut map = IndexMap::new();
        map.insert("this".to_string(), "another".to_string());
        map.insert("is".to_string(), "boy".to_string());
        map.insert("again".to_string(), "day".to_string());
        assert_eq!(replace_moustaches(txt, map), out);
    }
}
