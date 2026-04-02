//! Agent bundle deployment.
//!
//! Unpacks a tar.gz bundle containing an AGENT.toml and agent files,
//! validates the manifest, and prepares the rootfs layout for execution.

use crate::manifest::{self, AgentManifest};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum DeployError {
    InvalidBundle(String),
    ManifestError(String),
    IoError(std::io::Error),
    PathTraversal(String),
}

impl std::fmt::Display for DeployError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidBundle(msg) => write!(f, "invalid bundle: {msg}"),
            Self::ManifestError(msg) => write!(f, "manifest error: {msg}"),
            Self::IoError(e) => write!(f, "I/O error: {e}"),
            Self::PathTraversal(msg) => write!(f, "path traversal rejected: {msg}"),
        }
    }
}

impl std::error::Error for DeployError {}

impl From<std::io::Error> for DeployError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

/// Check that a tar entry path is safe (no ".." components, no absolute paths).
fn validate_entry_path(path: &Path) -> Result<(), DeployError> {
    let path_str = path.to_string_lossy();

    if path.is_absolute() {
        return Err(DeployError::PathTraversal(format!(
            "absolute path: {path_str}"
        )));
    }

    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(DeployError::PathTraversal(format!(
                "parent directory reference in: {path_str}"
            )));
        }
    }

    if path_str.contains('\0') {
        return Err(DeployError::PathTraversal(format!(
            "null byte in path: {path_str}"
        )));
    }

    Ok(())
}

/// Unpack an agent bundle (tar.gz bytes) and prepare it for execution.
///
/// Returns the parsed manifest on success.
///
/// Bundle layout expected:
/// ```text
/// <name>/AGENT.toml
/// <name>/bin/<name>          (for binary runtime)
/// <name>/main.py             (for python runtime)
/// <name>/requirements.txt    (optional, for python runtime)
/// ```
pub fn unpack_bundle(
    data: &[u8],
    agent_dir: &str,
    rootfs_base: &str,
) -> Result<AgentManifest, DeployError> {
    let decoder = flate2::read::GzDecoder::new(data);
    let mut archive = tar::Archive::new(decoder);

    let mut manifest_content: Option<String> = None;
    let mut entries_buf: Vec<(PathBuf, Vec<u8>)> = Vec::new();

    // First pass: read all entries into memory, validate paths, find AGENT.toml
    for entry_result in archive.entries().map_err(|e| {
        DeployError::InvalidBundle(format!("cannot read tar entries: {e}"))
    })? {
        let mut entry = entry_result.map_err(|e| {
            DeployError::InvalidBundle(format!("bad tar entry: {e}"))
        })?;

        let entry_path = entry.path().map_err(|e| {
            DeployError::InvalidBundle(format!("bad path in tar: {e}"))
        })?.into_owned();

        validate_entry_path(&entry_path)?;

        let mut content = Vec::new();
        entry.read_to_end(&mut content)?;

        // Check if this is the AGENT.toml (at <name>/AGENT.toml)
        if entry_path.file_name().is_some_and(|f| f == "AGENT.toml")
            && entry_path.components().count() == 2
        {
            manifest_content = Some(String::from_utf8(content.clone()).map_err(|e| {
                DeployError::ManifestError(format!("AGENT.toml is not valid UTF-8: {e}"))
            })?);
        }

        entries_buf.push((entry_path, content));
    }

    // Parse and validate the manifest
    let manifest_str = manifest_content.ok_or_else(|| {
        DeployError::InvalidBundle("AGENT.toml not found in bundle".to_string())
    })?;

    let agent_manifest = manifest::parse(&manifest_str).map_err(|e| {
        DeployError::ManifestError(e.to_string())
    })?;

    let name = &agent_manifest.agent.name;

    // Create rootfs layout
    let rootfs = format!("{rootfs_base}/{name}");
    let dirs = [
        format!("{rootfs}/agent/bin"),
        format!("{rootfs}/tmp"),
        format!("{rootfs}/proc"),
        format!("{rootfs}/sys"),
        format!("{rootfs}/dev"),
        format!("{rootfs}/etc"),
        format!("{rootfs}/run"),
        format!("{rootfs}/data"),
        format!("{rootfs}/data/output"),
    ];
    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    // Write /etc/resolv.conf and /etc/hostname
    std::fs::write(format!("{rootfs}/etc/resolv.conf"), "nameserver 1.1.1.1\n")?;
    std::fs::write(format!("{rootfs}/etc/hostname"), format!("{name}\n"))?;

    // Extract files to their destinations
    for (entry_path, content) in &entries_buf {
        let file_name = match entry_path.file_name() {
            Some(f) => f.to_string_lossy().to_string(),
            None => continue,
        };

        if content.is_empty() {
            continue; // skip directories
        }

        match file_name.as_str() {
            "AGENT.toml" => {
                // Save manifest to agent_dir
                let dest = format!("{agent_dir}/{name}.toml");
                if let Some(parent) = Path::new(&dest).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&dest, content)?;
            }
            f if f == *name => {
                // Binary: <name>/bin/<name> -> rootfs/<name>/agent/bin/<name>
                let dest = format!("{rootfs}/agent/bin/{name}");
                std::fs::write(&dest, content)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755))?;
                }
            }
            "main.py" => {
                let dest = format!("{rootfs}/agent/main.py");
                std::fs::write(&dest, content)?;
            }
            "requirements.txt" => {
                let dest = format!("{rootfs}/agent/requirements.txt");
                std::fs::write(&dest, content)?;
            }
            _ => {
                // Other files: preserve relative structure under agent/
                let dest = format!("{rootfs}/agent/{file_name}");
                std::fs::write(&dest, content)?;
            }
        }
    }

    Ok(agent_manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Runtime;
    use std::io::Write;

    /// Create a tar.gz bundle in memory with the given entries.
    fn make_bundle(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());

        for (path, data) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, path, &data[..]).unwrap();
        }

        let tar_data = builder.into_inner().unwrap();

        let mut gz_buf = Vec::new();
        let mut encoder =
            flate2::write::GzEncoder::new(&mut gz_buf, flate2::Compression::fast());
        encoder.write_all(&tar_data).unwrap();
        encoder.finish().unwrap();

        gz_buf
    }

    #[test]
    fn deploy_valid_bundle() {
        let manifest_toml = br#"
[agent]
name = "test-agent"
version = "1.0.0"
"#;

        let bundle = make_bundle(&[
            ("test-agent/AGENT.toml", manifest_toml),
            ("test-agent/bin/test-agent", b"#!/bin/sh\necho hello"),
        ]);

        let tmp = tempfile::tempdir().unwrap();
        let agent_dir = tmp.path().join("agents");
        let rootfs_base = tmp.path().join("rootfs");

        let manifest = unpack_bundle(
            &bundle,
            agent_dir.to_str().unwrap(),
            rootfs_base.to_str().unwrap(),
        )
        .unwrap();

        assert_eq!(manifest.agent.name, "test-agent");

        // Verify layout
        assert!(agent_dir.join("test-agent.toml").exists());
        assert!(rootfs_base.join("test-agent/agent/bin").is_dir());
        assert!(rootfs_base.join("test-agent/tmp").is_dir());
        assert!(rootfs_base.join("test-agent/proc").is_dir());
        assert!(rootfs_base.join("test-agent/etc/resolv.conf").exists());
        assert!(rootfs_base.join("test-agent/etc/hostname").exists());
        assert!(rootfs_base.join("test-agent/data/output").is_dir());

        let hostname = std::fs::read_to_string(rootfs_base.join("test-agent/etc/hostname")).unwrap();
        assert_eq!(hostname.trim(), "test-agent");
    }

    #[test]
    fn deploy_python_bundle() {
        let manifest_toml = br#"
[agent]
name = "py-bot"
runtime = "python"
"#;

        let bundle = make_bundle(&[
            ("py-bot/AGENT.toml", manifest_toml),
            ("py-bot/main.py", b"print('hello')"),
            ("py-bot/requirements.txt", b"requests==2.31.0"),
        ]);

        let tmp = tempfile::tempdir().unwrap();
        let agent_dir = tmp.path().join("agents");
        let rootfs_base = tmp.path().join("rootfs");

        let manifest = unpack_bundle(
            &bundle,
            agent_dir.to_str().unwrap(),
            rootfs_base.to_str().unwrap(),
        )
        .unwrap();

        assert_eq!(manifest.agent.runtime, Runtime::Python);
        assert!(rootfs_base.join("py-bot/agent/main.py").exists());
        assert!(rootfs_base.join("py-bot/agent/requirements.txt").exists());
    }

    #[test]
    fn reject_path_traversal() {
        // Build a tar with a path-traversal entry by manually crafting the header.
        // The tar crate's `append_data` rejects ".." so we write raw bytes.
        let mut tar_buf = Vec::new();

        // First, add a valid AGENT.toml
        let manifest_toml = b"[agent]\nname = \"evil\"\n";
        {
            let mut header = tar::Header::new_gnu();
            header.set_size(manifest_toml.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            // Write header
            let header_bytes: &[u8] = header.as_bytes();
            tar_buf.extend_from_slice(header_bytes);
            // Overwrite path in header (first 100 bytes)
            let path_bytes = b"evil/AGENT.toml";
            tar_buf[..path_bytes.len()].copy_from_slice(path_bytes);
            // Recompute checksum
            recompute_tar_cksum(&mut tar_buf[..512]);
            // Write data + padding
            tar_buf.extend_from_slice(manifest_toml);
            let padding = 512 - (manifest_toml.len() % 512);
            if padding < 512 {
                tar_buf.extend(std::iter::repeat(0u8).take(padding));
            }
        }

        // Second entry: path traversal
        let evil_data = b"root:x:0:0";
        {
            let mut header = tar::Header::new_gnu();
            header.set_size(evil_data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            let header_bytes: &[u8] = header.as_bytes();
            let header_start = tar_buf.len();
            tar_buf.extend_from_slice(header_bytes);
            // Overwrite path with traversal
            let evil_path = b"evil/../../../etc/passwd";
            tar_buf[header_start..header_start + evil_path.len()].copy_from_slice(evil_path);
            recompute_tar_cksum(&mut tar_buf[header_start..header_start + 512]);
            // Write data + padding
            tar_buf.extend_from_slice(evil_data);
            let padding = 512 - (evil_data.len() % 512);
            if padding < 512 {
                tar_buf.extend(std::iter::repeat(0u8).take(padding));
            }
        }

        // End-of-archive marker (two 512-byte zero blocks)
        tar_buf.extend(std::iter::repeat(0u8).take(1024));

        // Compress
        let mut gz_buf = Vec::new();
        let mut encoder =
            flate2::write::GzEncoder::new(&mut gz_buf, flate2::Compression::fast());
        encoder.write_all(&tar_buf).unwrap();
        encoder.finish().unwrap();

        let tmp = tempfile::tempdir().unwrap();
        let result = unpack_bundle(
            &gz_buf,
            tmp.path().join("agents").to_str().unwrap(),
            tmp.path().join("rootfs").to_str().unwrap(),
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, DeployError::PathTraversal(_)),
            "expected PathTraversal, got: {err}"
        );
    }

    /// Recompute the tar header checksum (bytes 148..156).
    fn recompute_tar_cksum(header: &mut [u8]) {
        // Per POSIX, the checksum is the sum of all bytes in the header,
        // treating the checksum field (bytes 148..156) as spaces (0x20).
        let mut sum: u32 = 0;
        for (i, &b) in header[..512].iter().enumerate() {
            if (148..156).contains(&i) {
                sum += 0x20u32;
            } else {
                sum += b as u32;
            }
        }
        let cksum = format!("{sum:06o}\0 ");
        header[148..156].copy_from_slice(cksum.as_bytes());
    }

    #[test]
    fn reject_missing_manifest() {
        let bundle = make_bundle(&[
            ("agent/bin/agent", b"binary data"),
        ]);

        let tmp = tempfile::tempdir().unwrap();
        let result = unpack_bundle(
            &bundle,
            tmp.path().join("agents").to_str().unwrap(),
            tmp.path().join("rootfs").to_str().unwrap(),
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, DeployError::InvalidBundle(_)),
            "expected InvalidBundle, got: {err}"
        );
    }
}
