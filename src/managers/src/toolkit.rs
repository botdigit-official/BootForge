//! Botdigit Rescue Toolkit: the repair tools a technician needs on every drive.
//!
//! Each tool is fetched from its **official** source at the latest release, verified
//! against the checksum the publisher provides, cached on this computer
//! (`~/Library/Caches/BootForge/toolkit`) and copied onto the drive with a clear name in
//! the boot menu.
//!
//! | Source         | How the latest version is found      | Checksum used                 |
//! |----------------|--------------------------------------|-------------------------------|
//! | SourceForge    | project RSS feed for the release dir | MD5 from the feed             |
//! | GitHub         | `releases/latest` API                | SHA-256 asset digest          |
//! | memtest.org    | version link on the home page        | `sha256sum.txt`               |
//! | Hiren's BootCD | fixed URL                            | SHA-256 on the download page  |

use md5::Md5;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

const UA: &str = "BootForge/0.1 (+https://github.com/botdigit-official/BootForge)";

#[derive(Debug, Clone, Copy)]
pub enum Source {
    /// Newest file in a SourceForge folder whose name ends with `suffix`.
    SourceForge { project: &'static str, path: &'static str, suffix: &'static str },
    /// Asset of the latest GitHub release: name contains all `include`, none of `exclude`,
    /// ends with `suffix`; `prefer` picks among several matches.
    GitHub {
        repo: &'static str,
        include: &'static [&'static str],
        exclude: &'static [&'static str],
        suffix: &'static str,
        prefer: Option<&'static str>,
    },
    /// MemTest86+ ISO (zipped) from memtest.org.
    MemtestOrg,
    /// Hiren's BootCD PE.
    HirensBootCd,
}

#[derive(Debug, Clone, Copy)]
pub struct ToolkitItem {
    pub id: &'static str,
    pub name: &'static str,
    /// What a technician uses it for.
    pub purpose: &'static str,
    pub default_selected: bool,
    pub approx_mb: u64,
    /// Name shown in the boot menu.
    pub menu_name: &'static str,
    pub source: Source,
}

pub static TOOLKIT: &[ToolkitItem] = &[
    ToolkitItem {
        id: "systemrescue",
        name: "SystemRescue",
        purpose: "All-in-one repair system: GParted, TestDisk/PhotoRec (recover deleted files), ddrescue (failing disks), SMART checks, reset Windows passwords (chntpw), network and SSH.",
        default_selected: true,
        approx_mb: 1450,
        menu_name: "Repair toolbox - SystemRescue",
        source: Source::SourceForge { project: "systemrescuecd", path: "/sysresccd-x86", suffix: "-amd64.iso" },
    },
    ToolkitItem {
        id: "clonezilla",
        name: "Clonezilla",
        purpose: "Back up, restore and clone whole disks or partitions (to another disk, USB or network share).",
        default_selected: true,
        approx_mb: 510,
        menu_name: "Backup and clone disks - Clonezilla",
        source: Source::SourceForge { project: "clonezilla", path: "/clonezilla_live_stable", suffix: "-amd64.iso" },
    },
    ToolkitItem {
        id: "gparted",
        name: "GParted Live",
        purpose: "Partition, resize, move and format any drive (NTFS, FAT32, exFAT, ext4, ...).",
        default_selected: true,
        approx_mb: 720,
        menu_name: "Partition and format drives - GParted",
        source: Source::SourceForge { project: "gparted", path: "/gparted-live-stable", suffix: "-amd64.iso" },
    },
    ToolkitItem {
        id: "shredos",
        name: "ShredOS",
        purpose: "Low-level secure erase of disks before reuse or disposal (nwipe, DoD/Gutmann/zero fill, verification).",
        default_selected: true,
        approx_mb: 360,
        menu_name: "Secure erase disks - ShredOS",
        source: Source::GitHub {
            repo: "PartialVolume/shredos.x86_64",
            include: &["_x86-64_"],
            exclude: &["lite", "plus-partition"],
            suffix: ".iso",
            prefer: None,
        },
    },
    ToolkitItem {
        id: "memtest",
        name: "MemTest86+",
        purpose: "Test RAM for faults (random crashes, blue screens, failed installs).",
        default_selected: true,
        approx_mb: 3,
        menu_name: "Memory test - MemTest86+",
        source: Source::MemtestOrg,
    },
    ToolkitItem {
        id: "rescuezilla",
        name: "Rescuezilla",
        purpose: "Point-and-click backup and restore with a desktop (reads Clonezilla backups). Easiest for beginners.",
        default_selected: false,
        approx_mb: 1600,
        menu_name: "Easy backup and restore - Rescuezilla",
        source: Source::GitHub {
            repo: "rescuezilla/rescuezilla",
            include: &["-64bit."],
            exclude: &[],
            suffix: ".iso",
            prefer: Some("noble"),
        },
    },
    ToolkitItem {
        id: "hirens",
        name: "Hiren's BootCD PE",
        purpose: "Windows PE toolkit: repair Windows boot, reset passwords, recover data, drivers, antivirus scans.",
        default_selected: false,
        approx_mb: 3300,
        menu_name: "Windows rescue - Hiren's BootCD PE",
        source: Source::HirensBootCd,
    },
];

/// Folder on the drive that holds toolkit images.
pub const DRIVE_FOLDER: &str = "images/rescue";

pub fn find_item(id: &str) -> Option<&'static ToolkitItem> {
    TOOLKIT.iter().find(|i| i.id == id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Checksum {
    Sha256(String),
    Md5(String),
}

/// A concrete download for the latest release of a tool.
#[derive(Debug, Clone)]
pub struct Resolved {
    pub url: String,
    pub file_name: String,
    pub size: Option<u64>,
    pub checksum: Checksum,
    /// The download is a zip containing the ISO.
    pub zipped: bool,
}

/// Progress callback: (status line, bytes done, bytes total, MB/s).
pub type ProgressFn<'a> = &'a mut dyn FnMut(&str, u64, u64, f64);

#[derive(Debug, Clone, Serialize)]
pub struct InstalledTool {
    pub id: String,
    pub file_name: String,
    pub reused_from_cache: bool,
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder().user_agent(UA).build().map_err(|e| e.to_string())
}

async fn get_text(url: &str) -> Result<String, String> {
    let resp = client()?.get(url).send().await.map_err(|e| format!("{url}: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("{url}: HTTP {}", resp.status()));
    }
    resp.text().await.map_err(|e| e.to_string())
}

// ── Resolvers (pure parsing is split out so it can be unit tested) ──

/// Pick the newest matching file from a SourceForge RSS feed (items are newest first).
pub fn parse_sourceforge_rss(rss: &str, suffix: &str) -> Option<Resolved> {
    for item in rss.split("<item>").skip(1) {
        let Some(start) = item.find("url=\"") else { continue };
        let rest = &item[start + 5..];
        let url = &rest[..rest.find('"')?];
        let file_path = url.trim_end_matches("/download");
        if !file_path.ends_with(suffix) {
            continue;
        }
        let file_name = file_path.rsplit('/').next()?.to_string();
        let size = item
            .find("filesize=\"")
            .and_then(|i| item[i + 10..].split('"').next())
            .and_then(|s| s.parse().ok());
        let md5 = item
            .find("algo=\"md5\">")
            .map(|i| item[i + 11..].chars().take_while(|c| c.is_ascii_hexdigit()).collect::<String>())
            .filter(|h| h.len() == 32)?;
        return Some(Resolved {
            url: url.to_string(),
            file_name,
            size,
            checksum: Checksum::Md5(md5.to_lowercase()),
            zipped: false,
        });
    }
    None
}

pub fn parse_github_release(
    json: &serde_json::Value,
    include: &[&str],
    exclude: &[&str],
    suffix: &str,
    prefer: Option<&str>,
) -> Option<Resolved> {
    let assets = json.get("assets")?.as_array()?;
    let matches: Vec<&serde_json::Value> = assets
        .iter()
        .filter(|a| {
            let n = a["name"].as_str().unwrap_or("");
            n.ends_with(suffix) && include.iter().all(|i| n.contains(i)) && !exclude.iter().any(|e| n.contains(e))
        })
        .collect();
    let pick = prefer
        .and_then(|p| matches.iter().find(|a| a["name"].as_str().unwrap_or("").contains(p)))
        .or_else(|| matches.first())?;
    let digest = pick["digest"].as_str()?.strip_prefix("sha256:")?.to_lowercase();
    Some(Resolved {
        url: pick["browser_download_url"].as_str()?.to_string(),
        file_name: pick["name"].as_str()?.to_string(),
        size: pick["size"].as_u64(),
        checksum: Checksum::Sha256(digest),
        zipped: false,
    })
}

/// Newest `download/vX.YZ/` version linked from memtest.org.
pub fn parse_memtest_version(html: &str) -> Option<String> {
    let mut versions: Vec<(Vec<u32>, String)> = html
        .match_indices("download/v")
        .filter_map(|(i, _)| {
            let v: String = html[i + 10..].chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
            let v = v.trim_end_matches('.').to_string();
            let key = v.split('.').filter_map(|p| p.parse().ok()).collect::<Vec<u32>>();
            (!key.is_empty()).then_some((key, v))
        })
        .collect();
    versions.sort();
    versions.pop().map(|(_, v)| v)
}

/// First 64-hex-digit string after `marker` in a page.
pub fn sha256_after(text: &str, marker: &str) -> Option<String> {
    let start = text.find(marker)?;
    let bytes = text[start..].as_bytes();
    let mut run = String::new();
    for &b in bytes {
        if (b as char).is_ascii_hexdigit() {
            run.push(b as char);
            if run.len() == 64 {
                return Some(run.to_lowercase());
            }
        } else {
            run.clear();
        }
    }
    None
}

pub async fn resolve(item: &ToolkitItem) -> Result<Resolved, String> {
    match item.source {
        Source::SourceForge { project, path, suffix } => {
            let rss = get_text(&format!("https://sourceforge.net/projects/{project}/rss?path={path}")).await?;
            parse_sourceforge_rss(&rss, suffix).ok_or_else(|| format!("No {suffix} release found for {}", item.name))
        }
        Source::GitHub { repo, include, exclude, suffix, prefer } => {
            let text = get_text(&format!("https://api.github.com/repos/{repo}/releases/latest")).await?;
            let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
            parse_github_release(&json, include, exclude, suffix, prefer)
                .ok_or_else(|| format!("No matching verified release asset for {}", item.name))
        }
        Source::MemtestOrg => {
            let home = get_text("https://www.memtest.org/").await?;
            let v = parse_memtest_version(&home).ok_or("memtest.org: no version found")?;
            let file = format!("mt86plus_{v}_x86_64.iso.zip");
            let sums = get_text(&format!("https://www.memtest.org/download/v{v}/sha256sum.txt")).await?;
            let sha = sums
                .lines()
                .find(|l| l.trim_end().ends_with(&file))
                .and_then(|l| l.split_whitespace().next())
                .ok_or("memtest.org: checksum not published")?;
            Ok(Resolved {
                url: format!("https://www.memtest.org/download/v{v}/{file}"),
                file_name: file,
                size: None,
                checksum: Checksum::Sha256(sha.to_lowercase()),
                zipped: true,
            })
        }
        Source::HirensBootCd => {
            let page = get_text("https://www.hirensbootcd.org/download/").await?;
            let sha = sha256_after(&page, "SHA-256").ok_or("hirensbootcd.org: checksum not found on the page")?;
            Ok(Resolved {
                url: "https://www.hirensbootcd.org/files/HBCD_PE_x64.iso".to_string(),
                file_name: "HBCD_PE_x64.iso".to_string(),
                size: None,
                checksum: Checksum::Sha256(sha),
                zipped: false,
            })
        }
    }
}

// ── Download, verify, cache ──

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("BootForge").join("toolkit")
}

fn checksum_label(c: &Checksum) -> String {
    match c {
        Checksum::Sha256(h) => format!("sha256:{h}"),
        Checksum::Md5(h) => format!("md5:{h}"),
    }
}

/// Download `r` into the cache (or reuse a verified copy). Returns the cached file.
pub async fn fetch_verified(r: &Resolved, progress: ProgressFn<'_>) -> Result<(PathBuf, bool), String> {
    use futures_util::StreamExt;

    let dir = cache_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = dir.join(&r.file_name);
    let marker = dir.join(format!("{}.verified", r.file_name));
    let label = checksum_label(&r.checksum);
    if dest.exists() && fs::read_to_string(&marker).map(|m| m.trim() == label).unwrap_or(false) {
        return Ok((dest, true));
    }

    let resp = client()?.get(&r.url).send().await.map_err(|e| format!("{}: {e}", r.url))?;
    if !resp.status().is_success() {
        return Err(format!("{}: HTTP {}", r.url, resp.status()));
    }
    let total = resp.content_length().or(r.size).unwrap_or(0);
    let part = dir.join(format!("{}.part", r.file_name));
    let mut out = fs::File::create(&part).map_err(|e| e.to_string())?;
    let mut sha = Sha256::new();
    let mut md5 = Md5::new();
    let mut done: u64 = 0;
    let started = Instant::now();
    let mut last_report = Instant::now();
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download interrupted: {e}"))?;
        out.write_all(&chunk).map_err(|e| e.to_string())?;
        match r.checksum {
            Checksum::Sha256(_) => sha.update(&chunk),
            Checksum::Md5(_) => md5.update(&chunk),
        }
        done += chunk.len() as u64;
        if last_report.elapsed().as_millis() > 400 {
            let mbps = done as f64 / 1e6 / started.elapsed().as_secs_f64().max(0.001);
            progress(&format!("Downloading {}", r.file_name), done, total, mbps);
            last_report = Instant::now();
        }
    }
    out.flush().map_err(|e| e.to_string())?;
    drop(out);

    let (got, want) = match &r.checksum {
        Checksum::Sha256(h) => (format!("{:x}", sha.finalize()), h.clone()),
        Checksum::Md5(h) => (format!("{:x}", md5.finalize()), h.clone()),
    };
    if got != want {
        let _ = fs::remove_file(&part);
        return Err(format!(
            "{} failed verification (expected {want}, got {got}). The download was discarded.",
            r.file_name
        ));
    }
    fs::rename(&part, &dest).map_err(|e| e.to_string())?;
    fs::write(&marker, &label).map_err(|e| e.to_string())?;
    Ok((dest, false))
}

/// Extract the single `.iso` from a zip next to it (uses the system `unzip`).
fn extract_iso_from_zip(zip: &Path) -> Result<PathBuf, String> {
    let out_dir = zip.with_extension("d");
    let _ = fs::create_dir_all(&out_dir);
    let status = std::process::Command::new("unzip")
        .args(["-o", "-j", "-q"])
        .arg(zip)
        .arg("-d")
        .arg(&out_dir)
        .status()
        .map_err(|e| format!("unzip not available: {e}"))?;
    if !status.success() {
        return Err(format!("Could not extract {}", zip.display()));
    }
    fs::read_dir(&out_dir)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("iso")))
        .ok_or_else(|| format!("No ISO inside {}", zip.display()))
}

fn copy_with_progress(src: &Path, dst: &Path, progress: ProgressFn<'_>) -> Result<(), String> {
    let total = fs::metadata(src).map_err(|e| e.to_string())?.len();
    if fs::metadata(dst).map(|m| m.len() == total).unwrap_or(false) {
        return Ok(());
    }
    let tmp = dst.with_extension("partial");
    let mut input = fs::File::open(src).map_err(|e| e.to_string())?;
    let mut output = fs::File::create(&tmp).map_err(|e| format!("Cannot write {}: {e}", tmp.display()))?;
    let mut buf = vec![0u8; 4 * 1024 * 1024];
    let mut done = 0u64;
    let started = Instant::now();
    loop {
        let n = input.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        output.write_all(&buf[..n]).map_err(|e| format!("Writing to the drive failed: {e}"))?;
        done += n as u64;
        let mbps = done as f64 / 1e6 / started.elapsed().as_secs_f64().max(0.001);
        progress(&format!("Copying {}", dst.file_name().unwrap_or_default().to_string_lossy()), done, total, mbps);
    }
    output.sync_all().map_err(|e| e.to_string())?;
    drop(output);
    fs::rename(&tmp, dst).map_err(|e| e.to_string())
}

/// Add a boot-menu name for an image in `/ventoy/ventoy.json` (keeps other entries).
pub fn set_menu_alias(drive: &Path, image_rel_path: &str, alias: &str) -> Result<(), String> {
    let path = drive.join("ventoy").join("ventoy.json");
    let mut cfg: serde_json::Value = fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .filter(|v: &serde_json::Value| v.is_object())
        .unwrap_or_else(|| serde_json::json!({}));
    let image = format!("/{}", image_rel_path.trim_start_matches('/'));
    let mut list: Vec<serde_json::Value> = cfg["menu_alias"].as_array().cloned().unwrap_or_default();
    list.retain(|e| e["image"].as_str() != Some(image.as_str()));
    list.push(serde_json::json!({ "image": image, "alias": alias }));
    cfg["menu_alias"] = serde_json::Value::Array(list);
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    fs::write(&path, serde_json::to_string_pretty(&cfg).unwrap_or_default()).map_err(|e| e.to_string())
}

/// Download (or reuse), verify and copy one tool onto a mounted BootForge drive.
pub async fn install_tool(item: &ToolkitItem, drive: &Path, progress: ProgressFn<'_>) -> Result<InstalledTool, String> {
    progress(&format!("Finding latest {}", item.name), 0, 0, 0.0);
    let resolved = resolve(item).await?;
    let (cached, reused) = fetch_verified(&resolved, progress).await?;
    let iso = if resolved.zipped { extract_iso_from_zip(&cached)? } else { cached };
    let file_name = iso.file_name().unwrap_or_default().to_string_lossy().to_string();

    let folder = drive.join(DRIVE_FOLDER);
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    // Remove older versions of the same tool so the menu shows one entry per tool.
    let stem_prefix: String = file_name.chars().take_while(|c| !c.is_ascii_digit()).collect();
    if stem_prefix.len() >= 4 {
        for old in fs::read_dir(&folder).map_err(|e| e.to_string())?.flatten() {
            let n = old.file_name().to_string_lossy().to_string();
            if n != file_name && n.starts_with(&stem_prefix) && n.ends_with(".iso") {
                let _ = fs::remove_file(old.path());
            }
        }
    }
    copy_with_progress(&iso, &folder.join(&file_name), progress)?;
    set_menu_alias(drive, &format!("{DRIVE_FOLDER}/{file_name}"), item.menu_name)?;
    Ok(InstalledTool { id: item.id.to_string(), file_name, reused_from_cache: reused })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sourceforge_rss_picks_newest_iso_with_md5() {
        let rss = r#"<rss><channel>
<item><title>/x/1.0/notes.txt</title><media:content url="https://sourceforge.net/projects/p/files/x/1.0/notes.txt/download" filesize="10"><media:hash algo="md5">00000000000000000000000000000000</media:hash></media:content></item>
<item><title>/x/1.8.1-6/gparted-live-1.8.1-6-amd64.iso</title><media:content xmlns:media="http://video.search.yahoo.com/mrss/" type="application/x-iso9660-image" url="https://sourceforge.net/projects/gparted/files/gparted-live-stable/1.8.1-6/gparted-live-1.8.1-6-amd64.iso/download" filesize="720371712"><media:hash algo="md5">8458737ba6b816e793a8061ed18b1e85</media:hash></media:content></item>
<item><media:content url="https://sourceforge.net/projects/gparted/files/gparted-live-stable/1.8.1-3/gparted-live-1.8.1-3-amd64.iso/download" filesize="1"><media:hash algo="md5">11111111111111111111111111111111</media:hash></media:content></item>
</channel></rss>"#;
        let r = parse_sourceforge_rss(rss, "-amd64.iso").unwrap();
        assert_eq!(r.file_name, "gparted-live-1.8.1-6-amd64.iso");
        assert_eq!(r.size, Some(720371712));
        assert_eq!(r.checksum, Checksum::Md5("8458737ba6b816e793a8061ed18b1e85".into()));
    }

    #[test]
    fn github_release_filters_and_requires_digest() {
        let json = serde_json::json!({"assets": [
            {"name": "shredos-x_32_x86-64_v1_lite.iso", "browser_download_url": "u1", "size": 1, "digest": "sha256:aa"},
            {"name": "shredos-x_32_x86-64_v1.iso.sha1", "browser_download_url": "u2", "size": 1, "digest": "sha256:bb"},
            {"name": "shredos-x_32_x86-64_v1.iso", "browser_download_url": "u3", "size": 358, "digest": "sha256:CC"},
            {"name": "shredos-x_32_i686_v1.iso", "browser_download_url": "u4", "size": 1, "digest": "sha256:dd"}
        ]});
        let r = parse_github_release(&json, &["_x86-64_"], &["lite", "plus-partition"], ".iso", None).unwrap();
        assert_eq!(r.url, "u3");
        assert_eq!(r.checksum, Checksum::Sha256("cc".into()));

        let rz = serde_json::json!({"assets": [
            {"name": "rescuezilla-2.6.2-64bit.oracular.iso", "browser_download_url": "a", "size": 1, "digest": "sha256:01"},
            {"name": "rescuezilla-2.6.2-64bit.noble.iso", "browser_download_url": "b", "size": 1, "digest": "sha256:02"}
        ]});
        assert_eq!(parse_github_release(&rz, &["-64bit."], &[], ".iso", Some("noble")).unwrap().url, "b");

        let no_digest = serde_json::json!({"assets": [{"name": "a.iso", "browser_download_url": "x", "size": 1}]});
        assert!(parse_github_release(&no_digest, &[], &[], ".iso", None).is_none());
    }

    #[test]
    fn memtest_version_and_page_checksums() {
        let html = r#"<a href="download/v7.20/x.zip"> <a href="download/v9.00/mt86plus_9.00.binaries.zip"> <a href="download/v8.10/y">"#;
        assert_eq!(parse_memtest_version(html).as_deref(), Some("9.00"));
        let page = "<p>ISO SHA-256</p><code>8c4c670c9c84d6c4b5a9c32e0aa5a55d8c23de851d259207d54679ea774c2498</code>";
        assert_eq!(
            sha256_after(page, "SHA-256").as_deref(),
            Some("8c4c670c9c84d6c4b5a9c32e0aa5a55d8c23de851d259207d54679ea774c2498")
        );
    }

    #[test]
    fn menu_alias_is_merged_not_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("ventoy")).unwrap();
        fs::write(dir.path().join("ventoy/ventoy.json"), r#"{"theme":{"file":"/t"},"menu_alias":[{"image":"/a.iso","alias":"A"}]}"#).unwrap();
        set_menu_alias(dir.path(), "images/rescue/b.iso", "B").unwrap();
        set_menu_alias(dir.path(), "images/rescue/b.iso", "B2").unwrap();
        let cfg: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join("ventoy/ventoy.json")).unwrap()).unwrap();
        assert_eq!(cfg["theme"]["file"], "/t");
        let aliases = cfg["menu_alias"].as_array().unwrap();
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[1]["alias"], "B2");
    }

    #[test]
    fn default_toolkit_is_reasonable() {
        let default_mb: u64 = TOOLKIT.iter().filter(|i| i.default_selected).map(|i| i.approx_mb).sum();
        assert!(default_mb < 4000, "default toolkit must fit an 8 GB drive with room to spare");
        let mut ids: Vec<_> = TOOLKIT.iter().map(|i| i.id).collect();
        ids.dedup();
        assert_eq!(ids.len(), TOOLKIT.len());
    }
}
