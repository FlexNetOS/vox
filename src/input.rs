//! Text input handling — CLI arguments or stdin pipe.

use std::io::{self, IsTerminal, Read};

use anyhow::{Result, bail};

pub fn read_text(args: &[String]) -> Result<String> {
    if !args.is_empty() {
        let text = args.join(" ");
        if text.trim().is_empty() {
            bail!("Text cannot be empty");
        }
        return Ok(text);
    }

    if io::stdin().is_terminal() {
        bail!("No text provided. Pass text as argument or pipe via stdin.");
    }

    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let text = buf.trim().to_string();
    if text.is_empty() {
        bail!("Text cannot be empty");
    }
    Ok(text)
}
