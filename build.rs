use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_assets()?;
    get_app_version()?;
    Ok(())
}


fn load_assets() -> Result<(), Box<dyn std::error::Error>> {
    embed_resource::compile("assets/app.rc", embed_resource::NONE).manifest_optional()?;
    Ok(())
}
fn get_app_version() -> Result<(), Box<dyn std::error::Error>> {
    let cargo_toml_path = Path::new("Cargo.toml");
    let content = fs::read_to_string(cargo_toml_path)?;
    let version = extract_version(&content)?;
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rustc-env=PROJECT_VERSION=v{version}");
    Ok(())
}
fn extract_version(toml_content: &str) -> Result<String, Box<dyn std::error::Error>> {
    // We are looking for the line: version = "X.Y.Z"
    let version_line = toml_content
        .lines()
        .find(|line| line.starts_with("version ="));

    match version_line {
        Some(line) => {
            // Example line: version = "1.2.3"
            // We split by '=' and then strip whitespace and quotes.
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() > 1 {
                // The version is usually the second part, which is quoted.
                let version_str = parts[1].trim().trim_matches('"');
                Ok(version_str.to_string())
            } else {
                Err("Could not find version definition in Cargo.toml".into())
            }
        }
        None => Err("Could not find 'version =' in Cargo.toml".into()),
    }
}
