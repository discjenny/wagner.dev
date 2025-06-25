use std::process::Command;
use std::fs;
use sha2::{Sha256, Digest};
use regex::Regex;
use glob::glob;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("running tailwind css...");
    let output = Command::new("tailwindcss")
        .args(["-i", "input.css", "-o", "static/computed.css", "--minify"])
        .output()?;

    if !output.status.success() {
        eprintln!("tailwind css failed:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err("Tailwind CSS build failed".into());
    }

    let css_content = fs::read("static/computed.css")?;
    let mut hasher = Sha256::new();
    hasher.update(&css_content);
    let hash = format!("{:x}", hasher.finalize());
    let short_hash = &hash[..8];

    let old_path = "static/computed.css";
    let new_filename = format!("computed.{}.css", short_hash);
    let new_path = format!("static/{}", new_filename);
    
    println!("renaming css file to: {}", new_filename);
    fs::rename(old_path, &new_path)?;

    println!("updating html references...");
    let html_pattern = "pages/**/*.html";
    let css_regex = Regex::new(r#"(?P<before>href=["'])(?P<path>/static/)computed\.(?:[a-f0-9]{8}\.)?css(?P<after>["'])"#)?;
    let replacement = format!("${{before}}${{path}}{}${{after}}", new_filename);

    let mut updated_files = 0;
    for entry in glob(html_pattern)? {
        let path = entry?;
        let content = fs::read_to_string(&path)?;
        let new_content = css_regex.replace_all(&content, replacement.as_str());
        
        if content != new_content {
            fs::write(&path, new_content.as_ref())?;
            println!("  updated: {}", path.display());
            updated_files += 1;
        }
    }

    if updated_files == 0 {
        println!("  no html files needed updating");
    }

    println!("cleaning up old css files...");
    let css_pattern = "static/computed.*.css";
    let mut cleaned_files = 0;
    
    for entry in glob(css_pattern)? {
        let path = entry?;
        let filename = path.file_name().unwrap().to_str().unwrap();
        
        if filename != new_filename {
            fs::remove_file(&path)?;
            println!("  removed: {}", filename);
            cleaned_files += 1;
        }
    }

    if cleaned_files == 0 {
        println!("  no old css files to clean up");
    }

    println!("done");

    Ok(())
} 