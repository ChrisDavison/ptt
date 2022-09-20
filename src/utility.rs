use anyhow::Result;
use std::io::{stdin, stdout, Write};

pub fn read_from_stdin(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    let _ = stdout().flush();
    let mut response = String::new();
    stdin().read_line(&mut response)?;
    Ok(response.trim().to_string())
}
