use anyhow::Result;
use sha2::{Digest, Sha256};
use std::io::{self, Write};

pub async fn run() -> Result<()> {
    let repo = "harshit-sandilya/mailcheck";
    let current = env!("CARGO_PKG_VERSION");
    println!("Current version: v{current}");
    print!("Checking latest release...");
    io::stdout().flush()?;

    let client = reqwest::Client::builder()
        .user_agent("mailcheck-updater")
        .build()?;

    let release: serde_json::Value = client
        .get(format!(
            "https://api.github.com/repos/{repo}/releases/latest"
        ))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let latest = release["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Could not read latest version"))?;
    println!(" latest: {latest}");

    if latest == format!("v{current}") {
        println!("Already up to date.");
        return Ok(());
    }

    // Detect platform
    let artifact = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "mailcheck-linux-x86_64",
        ("linux", "aarch64") => "mailcheck-linux-aarch64",
        ("macos", "x86_64") => "mailcheck-macos-x86_64",
        ("macos", "aarch64") => "mailcheck-macos-aarch64",
        (os, arch) => anyhow::bail!("Unsupported platform: {os}/{arch}"),
    };
    let url = format!("https://github.com/{repo}/releases/download/{latest}/{artifact}");
    println!("Downloading {url}...");
    let bytes = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let checksums_url =
        format!("https://github.com/{repo}/releases/download/{latest}/checksums.txt");
    let checksums = client
        .get(checksums_url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let expected = checksums
        .lines()
        .find_map(|line| {
            let mut fields = line.split_whitespace();
            let hash = fields.next()?;
            let name = fields.next()?;
            (name == artifact).then_some(hash)
        })
        .ok_or_else(|| anyhow::anyhow!("Release checksum missing for {artifact}"))?;
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if !actual.eq_ignore_ascii_case(expected) {
        anyhow::bail!("Checksum verification failed for {artifact}");
    }

    // Write next to current binary, then replace
    let current_exe = std::env::current_exe()?;
    let tmp = current_exe.with_extension("tmp");
    std::fs::write(&tmp, &bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
    }

    // Atomic replace — fails if no write permission
    if std::fs::rename(&tmp, &current_exe).is_err() {
        std::fs::remove_file(&tmp).ok();
        anyhow::bail!(
            "Permission denied replacing {}. Run with sudo or:\n  sudo mailcheck update",
            current_exe.display()
        );
    }

    println!("Updated to {latest} — you're good to go.");
    Ok(())
}
