//! BootForge — Universal Boot & Recovery Studio
//! macOS Native CLI & Desktop Web App Launcher.

mod web_ui;

use axum::extract::Query;
use axum::response::Html;
use axum::routing::{get, post};
use axum::{Json, Router};
use bootforge_analyzers::analyze_image;
use bootforge_core::types::{
    BootRepairResult, HardwareTelemetrySnapshot, RecoveryCategory, RecoveryToolItem, StorageDevice,
    UsbFileNode,
};
use bootforge_managers::device::{
    analyze_storage_device, mount_device_slice, open_mount_in_finder, scan_devices,
    unmount_device_slice, DeviceDetailsAnalysis,
};
use bootforge_managers::file_manager::{delete_usb_file, scan_usb_files};
use bootforge_managers::device::ensure_safe_write_target;
use bootforge_managers::format::{format_disk_macos, install_botdigit_boot_theme, sanitize_volume_label};
use bootforge_managers::hardware::capture_hardware_snapshot;
use bootforge_managers::repair::repair_bootloader_structure;
use bootforge_managers::writer::{copy_image_with_progress, TransferProgress};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};

static GLOBAL_PROGRESS: OnceLock<Mutex<TransferProgress>> = OnceLock::new();

fn get_progress() -> &'static Mutex<TransferProgress> {
    GLOBAL_PROGRESS.get_or_init(|| {
        Mutex::new(TransferProgress {
            is_running: false,
            is_completed: false,
            is_error: false,
            current_file: String::new(),
            file_index: 0,
            total_files: 0,
            bytes_written: 0,
            total_bytes: 0,
            percentage: 0.0,
            speed_mbps: 0.0,
            eta_seconds: 0,
            phase: "Ready".to_string(),
            message: "Waiting to start".to_string(),
            logs: Vec::new(),
        })
    })
}

#[derive(Parser, Debug)]
#[command(name = "bootforge")]
#[command(author = "Botdigit")]
#[command(version = "0.1.0")]
#[command(about = "Botdigit BootForge — Universal Boot & Recovery Studio (macOS, Windows, Linux)")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Port to bind the Web UI server
    #[arg(short, long, default_value_t = 4242)]
    port: u16,

    /// Do not automatically open the browser on launch
    #[arg(long)]
    no_open: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch the interactive Desktop Web UI on macOS (Default)
    Ui {
        #[arg(short, long, default_value_t = 4242)]
        port: u16,

        /// Do not automatically open the browser on launch
        #[arg(long)]
        no_open: bool,
    },
    /// List all physical and external storage devices detected on macOS
    Devices,
    /// Deep analyze a pen drive, external SSD, or storage device (e.g. disk9, /dev/disk6)
    AnalyzeDevice {
        /// Disk identifier or node path (e.g. disk9 or /dev/disk9)
        id: String,
    },
    /// Analyze an ISO, WIM, or disk image file and print compatibility report
    Analyze {
        /// Path to the ISO/image file
        path: PathBuf,
    },
    /// Format a USB drive into a Universal Botdigit BootForge Multi-Boot Drive
    Format {
        /// Disk identifier (e.g. disk5 or /dev/disk5)
        id: String,
        /// Volume label (default: BOOTFORGE)
        #[arg(short, long, default_value = "BOOTFORGE")]
        label: String,
        /// Skip the "type the disk name to confirm" prompt
        #[arg(long)]
        yes: bool,
        /// Also add the recommended Botdigit Rescue Toolkit (downloads ~2.8 GB once, then cached)
        #[arg(long)]
        toolkit: bool,
    },
    /// Explain, device by device, whether an image will boot and exactly how
    Guide {
        /// Path to the ISO/WIM/IMG/DMG file
        path: PathBuf,
        /// Only show one device class (e.g. legacy64, uefi, secureboot, intelmac, t2, applesilicon)
        #[arg(long)]
        device: Option<String>,
    },
    /// List the device classes BootForge plans for (old BIOS PCs to Apple Silicon)
    Targets,
    /// Botdigit Rescue Toolkit: list the repair tools, or add them to a mounted BootForge drive
    Toolkit {
        /// Mounted BootForge drive to install onto (e.g. /Volumes/BOOTFORGE). Omit to only list.
        #[arg(long)]
        drive: Option<PathBuf>,
        /// Comma-separated tool ids (default: the recommended set). Use `all` for every tool.
        #[arg(long)]
        tools: Option<String>,
    },
    /// Make a bootable macOS installer drive (separate drive; uses Apple's createinstallmedia)
    MacosInstaller {
        /// Path to "Install macOS <Name>.app" (download it from the App Store or `softwareupdate --fetch-full-installer`)
        #[arg(long)]
        app: PathBuf,
        /// Disk identifier of the USB drive (e.g. disk6). It will be erased.
        #[arg(long)]
        disk: String,
        /// Skip the "type the disk name to confirm" prompt
        #[arg(long)]
        yes: bool,
    },
    /// Run an automated headless QEMU boot test and print a structured report
    TestBoot {
        /// Disk identifier (e.g. disk9 or /dev/disk9)
        id: String,
        /// Test with UEFI firmware (OVMF/EDK2)
        #[arg(long)]
        uefi: bool,
        /// Timeout in seconds before the test is aborted (default: 60)
        #[arg(short, long, default_value_t = 60)]
        timeout: u64,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Devices) => {
            println!("🔍 Scanning macOS storage devices...\n");
            let devices = scan_devices();
            if devices.is_empty() {
                println!("No external storage devices found.");
            } else {
                for d in devices {
                    let lock_status = if d.is_system_drive {
                        "🔒 [SYSTEM - PROTECTED]"
                    } else if d.capacity > 100 * 1000 * 1000 * 1000 {
                        "⚠️ [LARGE DRIVE >100GB - CONFIRMATION REQUIRED]"
                    } else {
                        "✓ [READY FOR BOOT DRIVE]"
                    };
                    let size_gb = d.capacity as f64 / (1000.0 * 1000.0 * 1000.0);
                    println!(
                        "• {} ({} - {:.1} GB) -> /dev/{} [{}] {}",
                        d.vendor, d.model, size_gb, d.id, d.interface, lock_status
                    );
                }
            }
        }
        Some(Commands::AnalyzeDevice { id }) => {
            println!("🔍 Deep Analyzing Storage Device: {}...\n", id);
            match analyze_storage_device(&id) {
                Some(dev) => {
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!(" Device:           {} ({})", dev.model, dev.device_path);
                    println!(" Capacity:         {} ({} bytes)", dev.capacity_formatted, dev.capacity_bytes);
                    println!(" Partition Scheme: {}", dev.partition_scheme);
                    println!(" Bus & Speed:      {}", dev.interface_speed);
                    println!(" Mount Point:      {}", dev.mount_point.as_deref().unwrap_or("Not Mounted"));
                    println!(" Volume Label:     {}", dev.volume_label.as_deref().unwrap_or("N/A"));
                    println!(" Free Space:       {}", dev.free_space_formatted.as_deref().unwrap_or("N/A"));
                    println!(" Bootloader:       {}", dev.detected_bootloader);
                    println!(" Boot Status:      {}", dev.boot_readiness);
                    println!(" S.M.A.R.T. Health:{} (Wear: {}%, Temp: {:.1}°C)", dev.smart_status, dev.wear_percentage, dev.temperature_celsius);
                    println!(" Partitions ({}):", dev.partitions.len());
                    for p in &dev.partitions {
                        let boot_flag = if p.is_boot_partition { " [BOOTLOADER/ESP]" } else { "" };
                        println!(
                            "   #{} {} -> {} [{}] {} (Mount: {}){}",
                            p.number, p.identifier, p.size_formatted, p.filesystem, p.label, p.mount_point.as_deref().unwrap_or("None"), boot_flag
                        );
                    }
                    if !dev.existing_isos.is_empty() {
                        println!(" Installed OS Images on Drive ({}):", dev.existing_isos.len());
                        for iso in &dev.existing_isos {
                            println!("   {} {} ({}) -> {}", iso.icon, iso.file_name, iso.size_formatted, iso.detected_os);
                        }
                    }
                    println!(" Recommendation:   {}", dev.recommendation);
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                }
                None => {
                    eprintln!("❌ Storage device not found or unable to query: {}", id);
                }
            }
        }
        Some(Commands::Analyze { path }) => {
            println!("🔍 Analyzing image: {:?}...\n", path);
            match analyze_image(&path).await {
                Ok(analysis) => {
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!(" OS:            {}", analysis.os_info.display_name);
                    println!(" Architecture:  {}", analysis.architecture);
                    println!(" Format:        {}", analysis.format);
                    println!(" Size:          {:.2} GB", analysis.file_size as f64 / (1024.0 * 1024.0 * 1024.0));
                    println!(" UEFI Boot:     {}", if analysis.boot_info.uefi_capable { "Supported ✓" } else { "No" });
                    println!(" Secure Boot:   {:?}", analysis.boot_info.secure_boot);
                    println!(" Compatibility: {} (Score: {}/100)", analysis.compatibility.level, analysis.compatibility.score);
                    println!(" Strategy:      {:?}", analysis.boot_strategy);
                    println!(" Fingerprint:   {} (first 1 MiB; compare the publisher's checksum for the full file)", &analysis.hash.sha256[..16.min(analysis.hash.sha256.len())]);
                    for w in &analysis.warnings {
                        println!(" ⚠️  {w}");
                    }
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!(" Will it boot?");
                    for t in &analysis.device_guidance {
                        println!("  {} {:<36} {}", verdict_icon(t.verdict), t.device_name, t.summary);
                    }
                    println!("\n Run `bootforge guide \"{}\"` for step-by-step instructions.", path.display());
                }
                Err(e) => {
                    eprintln!("❌ Failed to analyze image: {}", e);
                }
            }
        }
        Some(Commands::Format { id, label, yes, toolkit }) => {
            let clean = id.trim_start_matches("/dev/").replace("rdisk", "disk");
            if let Err(reason) = ensure_safe_write_target(&clean) {
                eprintln!("❌ Refusing to write /dev/{clean}: {reason}");
                std::process::exit(1);
            }
            if !yes {
                println!("⚠️  ALL DATA on /dev/{clean} will be erased.");
                print!("Type the disk name ({clean}) to continue: ");
                use std::io::Write as _;
                std::io::stdout().flush().ok();
                let mut answer = String::new();
                std::io::stdin().read_line(&mut answer).ok();
                if answer.trim() != clean {
                    println!("Cancelled. Nothing was written.");
                    return Ok(());
                }
            }
            println!("⚡ Creating BootForge drive on /dev/{clean} (an administrator password prompt will appear)...");
            match format_disk_macos(&clean, "MBR", "ExFAT", &label) {
                Ok(msg) => {
                    println!("✅ {msg}");
                    if toolkit {
                        let mount = PathBuf::from(format!(
                            "/Volumes/{}",
                            bootforge_managers::format::sanitize_volume_label(&label)
                        ));
                        let ids = toolkit_ids(None);
                        if let Err(e) = install_toolkit_cli(&mount, &ids).await {
                            eprintln!("⚠️  Drive created, but the toolkit was not fully added: {e}");
                        }
                    }
                    println!("Next: copy .iso/.wim/.vhd/.img files onto the drive, then boot from it.");
                    println!("Run `bootforge guide <image.iso>` to see how to boot each image on your devices.");
                }
                Err(e) => {
                    eprintln!("❌ Failed to create drive on /dev/{clean}: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Guide { path, device }) => {
            let analysis = match analyze_image(&path).await {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("❌ Cannot read {}: {e}", path.display());
                    std::process::exit(1);
                }
            };
            println!("\n{} — {} ({})\n", analysis.os_info.display_name, analysis.architecture, analysis.file_name);
            for w in &analysis.warnings {
                println!("⚠️  {w}\n");
            }
            let filter = match device.as_deref() {
                None => None,
                Some(d) => match device_alias(d) {
                    Some(c) => Some(c),
                    None => {
                        eprintln!("Unknown device '{d}'. Use one of: old32, old64, uefi, secureboot, uefi32, intelmac, t2, applesilicon, arm64");
                        std::process::exit(2);
                    }
                },
            };
            for t in &analysis.device_guidance {
                if filter.is_some_and(|c| c != t.device) {
                    continue;
                }
                let profile = bootforge_analyzers::targets::DEVICE_PROFILES.iter().find(|p| p.class == t.device);
                println!("{} {} — {}", verdict_icon(t.verdict), t.device_name, t.verdict);
                if let Some(p) = profile {
                    println!("   e.g. {}", p.examples);
                }
                println!("   {}", t.summary);
                for (i, step) in t.steps.iter().enumerate() {
                    println!("   {}. {}", i + 1, step);
                }
                println!();
            }
        }
        Some(Commands::Toolkit { drive, tools }) => {
            let ids = toolkit_ids(tools.as_deref());
            match drive {
                None => {
                    println!("Botdigit Rescue Toolkit (✓ = recommended, added by default):\n");
                    for item in bootforge_managers::toolkit::TOOLKIT {
                        let latest = match bootforge_managers::toolkit::resolve(item).await {
                            Ok(r) => format!(
                                "{}{}",
                                r.file_name,
                                r.size.map(|s| format!(" ({:.0} MB)", s as f64 / 1e6)).unwrap_or_default()
                            ),
                            Err(e) => format!("unavailable: {e}"),
                        };
                        println!(
                            "{} {:<13} {:<20} {}\n    {}\n",
                            if item.default_selected { "✓" } else { " " },
                            item.id,
                            item.name,
                            latest,
                            item.purpose
                        );
                    }
                    println!("Add to a drive: bootforge toolkit --drive /Volumes/BOOTFORGE [--tools all|id,id]");
                }
                Some(mount) => {
                    if let Err(e) = install_toolkit_cli(&mount, &ids).await {
                        eprintln!("❌ {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
        Some(Commands::Targets) => {
            println!("Device classes BootForge plans for:\n");
            for p in bootforge_analyzers::targets::DEVICE_PROFILES {
                println!("• {}\n    examples: {}\n    boot:     {}\n", p.name, p.examples, p.how_to_boot);
            }
            println!("{}", bootforge_analyzers::targets::PC_BOOT_KEYS);
        }
        Some(Commands::MacosInstaller { app, disk, yes }) => {
            if let Err(e) = make_macos_installer(&app, &disk, yes) {
                eprintln!("❌ {e}");
                std::process::exit(1);
            }
        }
        Some(Commands::TestBoot { id, uefi, timeout }) => {
            let firmware = if uefi { "UEFI (OVMF)" } else { "Legacy BIOS (SeaBIOS)" };
            println!("\n╔════════════════════════════════════════════════════════════╗");
            println!("║   ⚡ BOTDIGIT BOOTFORGE — Headless Boot Test              ║");
            println!("╚════════════════════════════════════════════════════════════╝");
            println!("\n  Target:       /dev/{}", id);
            println!("  Firmware:     {}", firmware);
            println!("  Timeout:      {}s", timeout);
            println!("  Serial Log:   /tmp/bootforge_serial_disk{}.log", id.trim_start_matches("disk"));
            println!("\n⏳ Launching QEMU headless boot test...\n");

            match run_headless_boot_test(&id, uefi, timeout) {
                Ok(result) => {
                    let status_icon = if result.success { "✅ PASS" } else { "❌ FAIL" };
                    let time_display = if result.boot_time_ms < 1000 {
                        format!("{}ms", result.boot_time_ms)
                    } else {
                        format!("{:.1}s", result.boot_time_ms as f64 / 1000.0)
                    };

                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("  BOOT TEST REPORT");
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("  Result:       {}", status_icon);
                    println!("  Firmware:     {}", result.firmware_mode);
                    println!("  Boot Time:    {}", time_display);
                    println!("  Boot Stage:   {}", result.boot_stage_reached);
                    if let Some(ref err) = result.error {
                        println!("  Error:        {}", err);
                    }
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

                    // Print serial output summary (last 20 lines)
                    let lines: Vec<&str> = result.serial_output.lines().collect();
                    let tail = if lines.len() > 20 { &lines[lines.len()-20..] } else { &lines };
                    if !tail.is_empty() {
                        println!("\n  Serial Console (last {} lines):", tail.len());
                        println!("  ─────────────────────────────────────────────────────────");
                        for line in tail {
                            println!("  │ {}", line);
                        }
                        println!("  ─────────────────────────────────────────────────────────");
                    }
                    println!();
                }
                Err(e) => {
                    eprintln!("❌ Boot test failed: {}", e);
                }
            }
        }
        Some(Commands::Ui { port, no_open }) => {
            start_server(port, !(cli.no_open || no_open)).await?;
        }
        None => {
            start_server(cli.port, !cli.no_open).await?;
        }
    }

    Ok(())
}

async fn start_server(desired_port: u16, auto_open: bool) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(serve_ui))
        .route("/api/devices", get(api_devices))
        .route("/api/device-details", get(api_device_details))
        .route("/api/mount-device", post(api_mount_device))
        .route("/api/unmount-device", post(api_unmount_device))
        .route("/api/open-in-finder", post(api_open_in_finder))
        .route("/api/pick-file", get(api_pick_file))
        .route("/api/analyze", post(api_analyze))
        .route("/api/create-boot-drive", post(api_create_boot_drive))
        .route("/api/transfer-progress", get(api_transfer_progress))
        .route("/api/hardware-info", get(api_hardware_info))
        .route("/api/usb-files", get(api_usb_files))
        .route("/api/delete-usb-file", post(api_delete_usb_file))
        .route("/api/repair-bootloader", post(api_repair_bootloader))
        .route("/api/inject-legacy-mbr", post(api_inject_legacy_mbr))
        .route("/api/recovery-catalog", get(api_recovery_catalog))
        .route("/api/toolkit", get(api_toolkit))
        .route("/api/download-catalog", get(api_download_catalog))
        .route("/api/start-download", post(api_start_download))
        .route("/api/download-progress", get(api_download_progress))
        .route("/api/test-boot-vm", post(api_test_boot_vm))
        .route("/api/test-boot-headless", post(api_test_boot_headless));

    let mut port = desired_port;
    let mut listener = None;

    // Try finding an open port starting from desired_port
    for p in port..port + 20 {
        let addr = SocketAddr::from(([127, 0, 0, 1], p));
        if let Ok(l) = tokio::net::TcpListener::bind(addr).await {
            listener = Some(l);
            port = p;
            break;
        }
    }

    let listener = match listener {
        Some(l) => l,
        None => tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], desired_port))).await?,
    };

    let url = format!("http://127.0.0.1:{}", port);

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║       ⚡ BOTDIGIT BOOTFORGE UNIVERSAL STUDIO               ║");
    println!("║               One USB. Every Boot. (macOS)                 ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!("\n🚀 Botdigit BootForge Studio active at: {}", url);
    println!("🛡️  Safety Guard Active: Drives >100GB require confirmation, System drives protected.\n");

    if auto_open {
        let _ = open::that(&url);
    }

    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_ui() -> Html<&'static str> {
    Html(web_ui::INDEX_HTML)
}

async fn api_devices() -> Json<Vec<StorageDevice>> {
    let devices = scan_devices();
    Json(devices)
}

#[derive(Deserialize)]
struct DeviceDetailsQuery {
    id: String,
}

async fn api_device_details(Query(params): Query<DeviceDetailsQuery>) -> Json<Option<DeviceDetailsAnalysis>> {
    let details = analyze_storage_device(&params.id);
    Json(details)
}

#[derive(Deserialize)]
struct DeviceActionRequest {
    id: Option<String>,
    path: Option<String>,
}

#[derive(Serialize)]
struct ActionResponse {
    success: bool,
    message: String,
}

async fn api_mount_device(Json(req): Json<DeviceActionRequest>) -> Json<ActionResponse> {
    if let Some(id) = req.id {
        match mount_device_slice(&id) {
            Ok(mp) => Json(ActionResponse {
                success: true,
                message: format!("Mounted at {}", mp),
            }),
            Err(e) => Json(ActionResponse {
                success: false,
                message: e,
            }),
        }
    } else {
        Json(ActionResponse {
            success: false,
            message: "Missing device slice identifier".to_string(),
        })
    }
}

async fn api_unmount_device(Json(req): Json<DeviceActionRequest>) -> Json<ActionResponse> {
    if let Some(id) = req.id {
        match unmount_device_slice(&id) {
            Ok(()) => Json(ActionResponse {
                success: true,
                message: "Drive safely unmounted / ejected".to_string(),
            }),
            Err(e) => Json(ActionResponse {
                success: false,
                message: e,
            }),
        }
    } else {
        Json(ActionResponse {
            success: false,
            message: "Missing device identifier".to_string(),
        })
    }
}

async fn api_open_in_finder(Json(req): Json<DeviceActionRequest>) -> Json<ActionResponse> {
    if let Some(path) = req.path {
        match open_mount_in_finder(&path) {
            Ok(()) => Json(ActionResponse {
                success: true,
                message: "Opened in Finder".to_string(),
            }),
            Err(e) => Json(ActionResponse {
                success: false,
                message: e,
            }),
        }
    } else {
        Json(ActionResponse {
            success: false,
            message: "Missing mount path".to_string(),
        })
    }
}

async fn api_hardware_info() -> Json<HardwareTelemetrySnapshot> {
    let snapshot = capture_hardware_snapshot();
    Json(snapshot)
}

#[derive(Deserialize)]
struct UsbFilesQuery {
    mount_path: String,
}

async fn api_usb_files(Query(params): Query<UsbFilesQuery>) -> Json<Vec<UsbFileNode>> {
    let nodes = scan_usb_files(&params.mount_path);
    Json(nodes)
}

#[derive(Deserialize)]
struct DeleteFileRequest {
    mount_path: String,
    relative_path: String,
}

async fn api_delete_usb_file(Json(req): Json<DeleteFileRequest>) -> Json<ActionResponse> {
    match delete_usb_file(&req.mount_path, &req.relative_path) {
        Ok(()) => Json(ActionResponse {
            success: true,
            message: format!("Deleted {}", req.relative_path),
        }),
        Err(e) => Json(ActionResponse {
            success: false,
            message: e,
        }),
    }
}

#[derive(Deserialize)]
struct RepairBootloaderRequest {
    disk_id: String,
}

async fn api_repair_bootloader(Json(req): Json<RepairBootloaderRequest>) -> Json<BootRepairResult> {
    let result = repair_bootloader_structure(&req.disk_id);
    Json(result)
}

/// Short names accepted by `bootforge guide --device`.
fn device_alias(name: &str) -> Option<bootforge_core::types::DeviceClass> {
    use bootforge_core::types::DeviceClass::*;
    let n = name.to_lowercase().replace(['-', '_', ' '], "");
    Some(match n.as_str() {
        "old32" | "legacy32" | "legacybios32" | "32bit" | "netbook" => LegacyBios32,
        "old64" | "old" | "legacy" | "legacy64" | "legacybios64" | "bios" | "csm" => LegacyBios64,
        "uefi" | "uefipc" | "pc" => UefiPc,
        "secureboot" | "sb" | "modern" | "win11pc" => UefiSecureBoot,
        "uefi32" | "tablet" | "baytrail" => Uefi32,
        "intelmac" | "mac" | "imac" | "macbook" => IntelMac,
        "t2" | "intelmact2" => IntelMacT2,
        "applesilicon" | "m1" | "m2" | "m3" | "m4" | "arm64mac" => AppleSilicon,
        "arm64" | "arm" | "aarch64" | "snapdragon" => Arm64Uefi,
        _ => return None,
    })
}

/// Tool ids from `--tools` (None = recommended set, "all" = every tool).
fn toolkit_ids(spec: Option<&str>) -> Vec<String> {
    use bootforge_managers::toolkit::TOOLKIT;
    match spec.map(str::trim) {
        None | Some("") | Some("default") => {
            TOOLKIT.iter().filter(|i| i.default_selected).map(|i| i.id.to_string()).collect()
        }
        Some("all") => TOOLKIT.iter().map(|i| i.id.to_string()).collect(),
        Some(list) => list.split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect(),
    }
}

async fn install_toolkit_cli(mount: &std::path::Path, ids: &[String]) -> Result<(), String> {
    use bootforge_managers::toolkit::{find_item, install_tool};
    if !mount.join("ventoy").is_dir() {
        return Err(format!("{} is not a BootForge drive (no ventoy folder). Create one with `bootforge format`.", mount.display()));
    }
    let mut failed = Vec::new();
    for id in ids {
        let Some(item) = find_item(id) else {
            failed.push(format!("{id} (unknown tool)"));
            continue;
        };
        println!("🛠  {} — {}", item.name, item.purpose);
        let mut last_pct = 200u64;
        let mut report = |msg: &str, done: u64, total: u64, mbps: f64| {
            let pct = if total > 0 { done * 100 / total } else { 0 };
            if pct / 10 != last_pct / 10 || total == 0 {
                last_pct = pct;
                if total > 0 {
                    println!("   {msg}: {pct}% ({:.0}/{:.0} MB, {mbps:.1} MB/s)", done as f64 / 1e6, total as f64 / 1e6);
                } else {
                    println!("   {msg}");
                }
            }
        };
        match install_tool(item, mount, &mut report).await {
            Ok(t) => println!(
                "   ✅ {} verified and added{}",
                t.file_name,
                if t.reused_from_cache { " (from cache)" } else { "" }
            ),
            Err(e) => {
                println!("   ❌ {e}");
                failed.push(item.name.to_string());
            }
        }
    }
    if failed.is_empty() { Ok(()) } else { Err(format!("Not added: {}", failed.join(", "))) }
}

fn verdict_icon(v: bootforge_core::types::BootVerdict) -> &'static str {
    use bootforge_core::types::BootVerdict::*;
    match v {
        Works => "✅",
        WorksWithSteps => "🟡",
        NotSupported => "❌",
    }
}

/// Create a macOS installer USB with Apple's own `createinstallmedia`.
fn make_macos_installer(app: &std::path::Path, disk: &str, yes: bool) -> Result<(), String> {
    let tool = app.join("Contents/Resources/createinstallmedia");
    if !tool.exists() {
        return Err(format!(
            "{} is not a macOS installer app (no Contents/Resources/createinstallmedia).\n   \
             Get one with: softwareupdate --list-full-installers ; softwareupdate --fetch-full-installer --full-installer-version <version>",
            app.display()
        ));
    }
    let clean = disk.trim_start_matches("/dev/").replace("rdisk", "disk");
    ensure_safe_write_target(&clean)?;
    if !yes {
        println!("⚠️  ALL DATA on /dev/{clean} will be erased to create a macOS installer.");
        print!("Type the disk name ({clean}) to continue: ");
        use std::io::Write as _;
        std::io::stdout().flush().ok();
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer).ok();
        if answer.trim() != clean {
            println!("Cancelled. Nothing was written.");
            return Ok(());
        }
    }
    println!("1/2 Erasing /dev/{clean} as Mac OS Extended (GPT)...");
    let erase = Command::new("diskutil")
        .args(["eraseDisk", "JHFS+", "MACINSTALL", "GPT", &clean])
        .status()
        .map_err(|e| e.to_string())?;
    if !erase.success() {
        return Err(format!("diskutil eraseDisk failed on /dev/{clean}"));
    }
    println!("2/2 Running Apple's createinstallmedia (takes 10–30 minutes; enter your password if asked)...");
    let status = Command::new("sudo")
        .arg(&tool)
        .args(["--volume", "/Volumes/MACINSTALL", "--nointeraction"])
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("createinstallmedia failed (see the messages above)".to_string());
    }
    println!("✅ macOS installer drive ready.");
    println!("   Intel Mac: hold Option (⌥) at power on.  T2 Mac: first allow external boot in Startup Security Utility.");
    println!("   Apple Silicon: shut down, hold the power button until startup options appear.");
    Ok(())
}

fn resolve_write_mbr_core_path() -> PathBuf {
    bootforge_managers::format::resolve_disk_helper_path()
}

#[derive(Deserialize)]
struct InjectLegacyMbrRequest {
    disk_id: String,
}

#[derive(Serialize)]
struct InjectLegacyMbrResponse {
    success: bool,
    message: String,
}

async fn api_inject_legacy_mbr(Json(req): Json<InjectLegacyMbrRequest>) -> Json<InjectLegacyMbrResponse> {
    let clean_id = req.disk_id.trim_start_matches("/dev/").replace("rdisk", "disk");

    let result = tokio::task::spawn_blocking(move || -> Result<String, String> {
        ensure_safe_write_target(&clean_id)?;
        let bin = resolve_write_mbr_core_path();
        if !bin.exists() {
            return Err(format!(
                "Disk helper not found at {}. Build it with: cargo build --release --bin write_mbr_core",
                bin.display()
            ));
        }
        #[cfg(target_os = "macos")]
        {
            bootforge_managers::format::run_disk_helper_as_admin(
                &[&format!("/dev/{clean_id}")],
                &format!("BootForge needs to re-install the Legacy BIOS boot code on /dev/{clean_id}."),
            )
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err("Only supported on macOS in this build".to_string())
        }
    })
    .await
    .unwrap_or_else(|e| Err(e.to_string()));

    match result {
        Ok(msg) => Json(InjectLegacyMbrResponse { success: true, message: msg.trim().to_string() }),
        Err(e) => Json(InjectLegacyMbrResponse { success: false, message: e }),
    }
}


#[derive(Serialize)]
struct ToolkitEntry {
    id: &'static str,
    name: &'static str,
    purpose: &'static str,
    default_selected: bool,
    approx_mb: u64,
}

async fn api_toolkit() -> Json<Vec<ToolkitEntry>> {
    Json(
        bootforge_managers::toolkit::TOOLKIT
            .iter()
            .map(|i| ToolkitEntry {
                id: i.id,
                name: i.name,
                purpose: i.purpose,
                default_selected: i.default_selected,
                approx_mb: i.approx_mb,
            })
            .collect(),
    )
}

async fn api_recovery_catalog() -> Json<Vec<RecoveryToolItem>> {
    let catalog = vec![
        RecoveryToolItem {
            id: "clonezilla".to_string(),
            name: "Clonezilla Live".to_string(),
            category: RecoveryCategory::DiskRepair,
            icon: "💽".to_string(),
            description: "Bare-metal partition and disk imaging, cloning, and backup solution.".to_string(),
            size_estimate: "410 MB".to_string(),
            default_filename: "clonezilla-live-3.1.2-amd64.iso".to_string(),
            is_available: true,
            download_url: Some("https://clonezilla.org/downloads.php".to_string()),
        },
        RecoveryToolItem {
            id: "gparted".to_string(),
            name: "GParted Live".to_string(),
            category: RecoveryCategory::DiskRepair,
            icon: "✂️".to_string(),
            description: "Partition editor for creating, reorganizing, resizing, and deleting disk partitions.".to_string(),
            size_estimate: "520 MB".to_string(),
            default_filename: "gparted-live-1.6.0-1-amd64.iso".to_string(),
            is_available: true,
            download_url: Some("https://gparted.org/download.php".to_string()),
        },
        RecoveryToolItem {
            id: "memtest".to_string(),
            name: "MemTest86+".to_string(),
            category: RecoveryCategory::MemoryDiagnostics,
            icon: "🧠".to_string(),
            description: "Thorough, stand-alone memory test for x86 and x86_64 architecture computers.".to_string(),
            size_estimate: "15 MB".to_string(),
            default_filename: "memtest86-usb.iso".to_string(),
            is_available: true,
            download_url: Some("https://www.memtest.org/".to_string()),
        },
        RecoveryToolItem {
            id: "testdisk".to_string(),
            name: "TestDisk & PhotoRec".to_string(),
            category: RecoveryCategory::DataRecovery,
            icon: "💾".to_string(),
            description: "Powerful free data recovery software to recover lost partitions and undelete files.".to_string(),
            size_estimate: "650 MB".to_string(),
            default_filename: "systemrescue-11.01-amd64.iso".to_string(),
            is_available: true,
            download_url: Some("https://www.cgsecurity.org/".to_string()),
        },
        RecoveryToolItem {
            id: "chntpw".to_string(),
            name: "Offline NT Password Editor".to_string(),
            category: RecoveryCategory::AccountPassword,
            icon: "🔐".to_string(),
            description: "Utility to reset the password of any user with a local account on Windows.".to_string(),
            size_estimate: "18 MB".to_string(),
            default_filename: "cd140201.iso".to_string(),
            is_available: true,
            download_url: Some("http://pogostick.net/~pnh/ntpasswd/".to_string()),
        },
        RecoveryToolItem {
            id: "hwinfo".to_string(),
            name: "Hardware Burn-In & Diagnostics".to_string(),
            category: RecoveryCategory::HardwareTesting,
            icon: "🖥".to_string(),
            description: "Comprehensive hardware stress testing and component inspection toolkit.".to_string(),
            size_estimate: "290 MB".to_string(),
            default_filename: "hw-diag-live.iso".to_string(),
            is_available: true,
            download_url: None,
        },
    ];
    Json(catalog)
}

use bootforge_managers::download::{
    download_os_image, get_curated_download_catalog, DownloadItem, DownloadStatus,
};

static GLOBAL_DOWNLOAD_PROGRESS: OnceLock<Mutex<DownloadStatus>> = OnceLock::new();

fn get_download_progress() -> &'static Mutex<DownloadStatus> {
    GLOBAL_DOWNLOAD_PROGRESS.get_or_init(|| {
        Mutex::new(DownloadStatus {
            is_downloading: false,
            is_completed: false,
            is_error: false,
            item_id: String::new(),
            file_name: String::new(),
            saved_path: String::new(),
            bytes_downloaded: 0,
            total_bytes: 0,
            percentage: 0.0,
            speed_mbps: 0.0,
            eta_seconds: 0,
            message: "Idle".to_string(),
        })
    })
}

async fn api_download_catalog() -> Json<Vec<DownloadItem>> {
    Json(get_curated_download_catalog())
}

#[derive(Deserialize)]
struct StartDownloadRequest {
    item_id: String,
}

async fn api_start_download(Json(req): Json<StartDownloadRequest>) -> Json<ActionResponse> {
    let catalog = get_curated_download_catalog();
    if let Some(item) = catalog.into_iter().find(|i| i.id == req.item_id) {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/botdigit".to_string());
        let target_dir = PathBuf::from(home).join("Downloads").join("BootForge_ISOs");
        let item_name = item.name.clone();

        tokio::spawn(async move {
            let _ = download_os_image(item, target_dir, |status| {
                let mut p = get_download_progress().lock().unwrap();
                *p = status;
            }).await;
        });

        Json(ActionResponse {
            success: true,
            message: format!("Started downloading {} in background", item_name),
        })
    } else {
        Json(ActionResponse {
            success: false,
            message: "Download item not found in catalog".to_string(),
        })
    }
}

async fn api_download_progress() -> Json<DownloadStatus> {
    let p = get_download_progress().lock().unwrap();
    Json(p.clone())
}

use bootforge_managers::vm::{launch_live_vm_test, run_headless_boot_test};

#[derive(Deserialize)]
struct TestVmRequest {
    disk_id: String,
    #[serde(default)]
    is_uefi: bool,
}

async fn api_test_boot_vm(Json(req): Json<TestVmRequest>) -> Json<ActionResponse> {
    match launch_live_vm_test(&req.disk_id, req.is_uefi) {
        Ok(msg) => Json(ActionResponse {
            success: true,
            message: msg,
        }),
        Err(e) => Json(ActionResponse {
            success: false,
            message: e,
        }),
    }
}

use bootforge_managers::vm::HeadlessBootTestResult;

#[derive(Deserialize)]
struct HeadlessBootTestRequest {
    disk_id: String,
    #[serde(default)]
    is_uefi: bool,
    #[serde(default = "default_boot_timeout")]
    timeout_secs: u64,
}

fn default_boot_timeout() -> u64 {
    60
}

#[derive(Serialize)]
struct HeadlessBootTestResponse {
    success: bool,
    result: Option<HeadlessBootTestResult>,
    error: Option<String>,
}

/// Run a headless QEMU boot test and return structured JSON telemetry.
/// POST /api/test-boot-headless
/// { "disk_id": "disk9", "is_uefi": true, "timeout_secs": 60 }
async fn api_test_boot_headless(Json(req): Json<HeadlessBootTestRequest>) -> Json<HeadlessBootTestResponse> {
    let timeout = req.timeout_secs.clamp(10, 300);

    match run_headless_boot_test(&req.disk_id, req.is_uefi, timeout) {
        Ok(result) => Json(HeadlessBootTestResponse {
            success: result.success,
            result: Some(result),
            error: None,
        }),
        Err(e) => Json(HeadlessBootTestResponse {
            success: false,
            result: None,
            error: Some(e),
        }),
    }
}

#[derive(Deserialize)]
struct CreateBootDriveRequest {
    disk_id: String,
    mode: String,
    partition_scheme: String,
    filesystem: String,
    volume_label: String,
    #[serde(default)]
    theme_name: String,
    #[serde(default)]
    isos: Vec<IsoItemPayload>,
    /// Botdigit Rescue Toolkit tool ids to add.
    #[serde(default)]
    toolkit: Vec<String>,
}

#[derive(Deserialize, Clone)]
struct IsoItemPayload {
    #[serde(default)]
    file_path: String,
    #[serde(default)]
    file_name: String,
    #[serde(default)]
    display_name: String,
}

#[derive(Serialize)]
struct CreateBootDriveResponse {
    success: bool,
    message: String,
    mount_point: Option<String>,
}

async fn api_create_boot_drive(Json(req): Json<CreateBootDriveRequest>) -> Json<CreateBootDriveResponse> {
    let clean_id = req.disk_id.trim_start_matches("/dev/").to_string();
    let isos = req.isos.clone();
    let toolkit_ids = req.toolkit.clone();
    let mode = req.mode.clone();
    let scheme = req.partition_scheme.clone();
    let fs_type = req.filesystem.clone();
    let label = sanitize_volume_label(&req.volume_label);

    let task_clean_id = clean_id.clone();
    let task_label = label.clone();
    let task_theme = if req.theme_name.is_empty() {
        "Botdigit BootForge Universal Multi-Boot Studio".to_string()
    } else {
        req.theme_name.clone()
    };

    // Spawn synchronous task to track live progress
    tokio::task::spawn_blocking(move || {
        let clean_id = task_clean_id;
        let label = task_label;
        let theme_title = task_theme;
        let mut p = get_progress().lock().unwrap();
        p.is_running = true;
        p.is_completed = false;
        p.is_error = false;
        p.percentage = 5.0;
        p.phase = "Initializing Botdigit Multi-Boot Engine...".to_string();
        p.logs.clear();
        p.logs.push(format!("[{}] Initializing target drive /dev/{}", chrono_time(), clean_id));
        drop(p);

        // Step 1: Format disk if fresh multiboot
        if mode == "fresh_multiboot" {
            let mut p = get_progress().lock().unwrap();
            p.percentage = 10.0;
            p.phase = format!("Erasing and writing boot structures (name {})… approve the password prompt", label);
            p.logs.push(format!(
                "[{}] Erasing /dev/{} -> MBR (BIOS boot code) + exFAT data + 32 MiB UEFI partition",
                chrono_time(),
                clean_id
            ));
            drop(p);

            match format_disk_macos(&clean_id, &scheme, &fs_type, &label) {
                Ok(msg) => {
                    let mut p = get_progress().lock().unwrap();
                    p.percentage = 25.0;
                    for line in msg.lines() {
                        p.logs.push(format!("[{}] ✓ {}", chrono_time(), line));
                    }
                    drop(p);
                }
                Err(err) => {
                    let mut p = get_progress().lock().unwrap();
                    p.is_running = false;
                    p.is_error = true;
                    p.message = format!("Format failed: {}", err);
                    p.logs.push(format!("[{}] ❌ Format error: {}", chrono_time(), err));
                    return;
                }
            }
        }

        // Step 2: Query mount point
        let mount_dir = if let Some(info) = analyze_storage_device(&clean_id) {
            info.mount_point.unwrap_or_else(|| format!("/Volumes/{}", label))
        } else {
            format!("/Volumes/{}", label)
        };

        let mount_path = PathBuf::from(&mount_dir);
        let mut failed: Vec<String> = Vec::new();

        // Step 3: Install BootForge configuration (theme, options, F6 tools page)
        if mount_path.exists() {
            let mut p = get_progress().lock().unwrap();
            p.percentage = 35.0;
            p.phase = "Installing BootForge menu theme and tools page...".to_string();
            p.logs.push(format!("[{}] Writing /ventoy/ theme, options and F6 tools page on {}", chrono_time(), mount_dir));
            drop(p);

            if let Err(e) = install_botdigit_boot_theme(&mount_path, &theme_title, &label) {
                failed.push(format!("configuration ({e})"));
            }

            // Step 4: Physically copy selected ISOs to destination
            let total_isos = isos.len();
            for (idx, iso) in isos.iter().enumerate() {
                let category = image_folder_for(&iso.file_name);

                let target_dir = mount_path.join("images").join(category);
                let _ = fs::create_dir_all(&target_dir);
                let dest_file = target_dir.join(&iso.file_name);

                let mut p = get_progress().lock().unwrap();
                p.file_index = idx + 1;
                p.total_files = total_isos;
                p.current_file = iso.display_name.clone();
                p.phase = format!("Streaming payload: {}...", iso.display_name);
                p.logs.push(format!("[{}] Copying payload: {} -> /images/{}/{}", chrono_time(), iso.display_name, category, iso.file_name));
                drop(p);

                let src = PathBuf::from(&iso.file_path);
                let outcome = if !src.is_file() {
                    Err(format!("source file not found: {}", iso.file_path))
                } else if dest_file.exists()
                    && fs::metadata(&dest_file).map(|m| m.len()).ok() == fs::metadata(&src).map(|m| m.len()).ok()
                {
                    Ok("already on the drive".to_string())
                } else {
                    copy_image_with_progress(&src, &dest_file, |written, total, speed, eta| {
                        let mut p = get_progress().lock().unwrap();
                        let base_pct = 40.0 + ((idx as f32 / total_isos as f32) * 55.0);
                        let sub_pct = if total > 0 { (written as f32 / total as f32) * (55.0 / total_isos as f32) } else { 0.0 };
                        p.percentage = (base_pct + sub_pct).min(98.0);
                        p.bytes_written = written;
                        p.total_bytes = total;
                        p.speed_mbps = speed;
                        p.eta_seconds = eta;
                    })
                    .map(|_| "copied".to_string())
                    .map_err(|e| e.to_string())
                };

                let mut p = get_progress().lock().unwrap();
                match outcome {
                    Ok(how) => p.logs.push(format!("[{}] ✓ {}: {}", chrono_time(), iso.display_name, how)),
                    Err(e) => {
                        failed.push(iso.display_name.clone());
                        p.logs.push(format!("[{}] ❌ {}: {}", chrono_time(), iso.display_name, e));
                        let _ = fs::remove_file(&dest_file);
                    }
                }
            }
        } else {
            failed.push(format!("drive not mounted at {}", mount_dir));
        }

        // Step 5: Botdigit Rescue Toolkit (download once, verify, cache, copy)
        if mount_path.exists() && !toolkit_ids.is_empty() {
            let rt = tokio::runtime::Handle::current();
            let count = toolkit_ids.len();
            for (i, id) in toolkit_ids.iter().enumerate() {
                let Some(item) = bootforge_managers::toolkit::find_item(id) else { continue };
                {
                    let mut p = get_progress().lock().unwrap();
                    p.logs.push(format!("[{}] 🛠 {}: {}", chrono_time(), item.name, item.purpose));
                }
                let mut report = |msg: &str, done: u64, total: u64, mbps: f64| {
                    let mut p = get_progress().lock().unwrap();
                    let frac = if total > 0 { done as f32 / total as f32 } else { 0.0 };
                    p.phase = format!("Rescue Toolkit {}/{}: {}", i + 1, count, msg);
                    p.current_file = item.name.to_string();
                    p.percentage = (60.0 + 38.0 * ((i as f32 + frac) / count as f32)).min(98.0);
                    p.bytes_written = done;
                    p.total_bytes = total;
                    p.speed_mbps = mbps as f32;
                    p.eta_seconds = if mbps > 0.0 && total > done { ((total - done) as f64 / 1e6 / mbps) as u64 } else { 0 };
                };
                let result = rt.block_on(bootforge_managers::toolkit::install_tool(item, &mount_path, &mut report));
                let mut p = get_progress().lock().unwrap();
                match result {
                    Ok(t) => p.logs.push(format!(
                        "[{}] ✓ {} verified and added ({}{})",
                        chrono_time(),
                        item.name,
                        t.file_name,
                        if t.reused_from_cache { ", from cache" } else { "" }
                    )),
                    Err(e) => {
                        failed.push(item.name.to_string());
                        p.logs.push(format!("[{}] ❌ {}: {}", chrono_time(), item.name, e));
                    }
                }
            }
        }

        if !failed.is_empty() {
            let mut p = get_progress().lock().unwrap();
            p.is_running = false;
            p.is_error = true;
            p.percentage = 100.0;
            p.phase = "Finished with errors".to_string();
            p.message = format!("Not copied: {}", failed.join(", "));
            return;
        }

        let mut p = get_progress().lock().unwrap();
        p.is_running = false;
        p.is_completed = true;
        p.percentage = 100.0;
        p.speed_mbps = 0.0;
        p.eta_seconds = 0;
        p.phase = "✨ BootForge drive ready".to_string();
        p.message = "Drive is ready. Boot it with your computer's boot-menu key.".to_string();
        p.logs.push(format!("[{}] ✅ Synchronization completed. Drive is ready to boot.", chrono_time()));
    });

    let mount_dir = if let Some(info) = analyze_storage_device(&clean_id) {
        info.mount_point.unwrap_or_else(|| format!("/Volumes/{}", label))
    } else {
        format!("/Volumes/{}", label)
    };

    Json(CreateBootDriveResponse {
        success: true,
        message: format!("Botdigit BootForge multi-boot configured on {}", mount_dir),
        mount_point: Some(mount_dir),
    })
}

async fn api_transfer_progress() -> Json<TransferProgress> {
    let p = get_progress().lock().unwrap();
    Json(p.clone())
}

/// Suggested folder on the drive for an image (the boot menu finds images anywhere).
fn image_folder_for(file_name: &str) -> &'static str {
    use bootforge_core::types::OsFamily;
    let info = bootforge_analyzers::os_detect::identify_os_from_label(file_name);
    match info.family {
        OsFamily::Windows => "windows",
        OsFamily::Linux | OsFamily::Bsd => "linux",
        OsFamily::MacOs => "macos-tools",
        OsFamily::Recovery | OsFamily::Utility | OsFamily::Firmware => "rescue",
        OsFamily::Unknown => "other",
    }
}

fn chrono_time() -> String {
    let now = chrono::Local::now();
    now.format("%H:%M:%S").to_string()
}

#[derive(Serialize)]
struct PickFileResponse {
    success: bool,
    path: Option<String>,
    paths: Vec<String>,
    cancelled: bool,
    error: Option<String>,
}

async fn api_pick_file() -> Json<PickFileResponse> {
    let result = tokio::task::spawn_blocking(|| {
        #[cfg(target_os = "macos")]
        {
            let script = r#"set picked to choose file with prompt "Choose operating system images" of type {"iso", "wim", "img", "vhd", "vhdx", "dmg", "raw", "bin", "efi"} default location (path to downloads folder) with multiple selections allowed
set out to ""
repeat with f in picked
set out to out & POSIX path of f & linefeed
end repeat
return out"#;
            let output = Command::new("osascript")
                .args(["-e", script])
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        if !path.is_empty() {
                            return Ok(Some(path));
                        }
                    }
                    Ok(None)
                }
                Err(e) => Err(e.to_string()),
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            Ok::<Option<String>, String>(None)
        }
    })
    .await;

    match result {
        Ok(Ok(Some(path))) => Json(PickFileResponse {
            success: true,
            paths: path.lines().map(str::trim).filter(|l| !l.is_empty()).map(String::from).collect(),
            path: path.lines().next().map(|l| l.trim().to_string()),
            cancelled: false,
            error: None,
        }),
        Ok(Ok(None)) => Json(PickFileResponse {
            paths: vec![],
            success: false,
            path: None,
            cancelled: true,
            error: None,
        }),
        Ok(Err(e)) => Json(PickFileResponse {
            paths: vec![],
            success: false,
            path: None,
            cancelled: false,
            error: Some(e),
        }),
        Err(e) => Json(PickFileResponse {
            paths: vec![],
            success: false,
            path: None,
            cancelled: false,
            error: Some(e.to_string()),
        }),
    }
}

#[derive(Deserialize)]
struct AnalyzeRequest {
    path: String,
}

#[derive(Serialize)]
struct AnalyzeResponse {
    success: bool,
    file_path: String,
    file_name: String,
    file_size_bytes: u64,
    file_size_formatted: String,
    display_name: String,
    arch: String,
    format: String,
    uefi_capable: bool,
    secure_boot: String,
    compat_level: String,
    score: u32,
    boot_strategy: String,
    sha256_preview: String,
    uefi_x64_score: u8,
    bios_score: u8,
    secure_boot_score: u8,
    arm64_score: u8,
    #[serde(default)]
    warnings: Vec<String>,
    #[serde(default)]
    device_guidance: Vec<bootforge_core::types::TargetVerdict>,
    error: Option<String>,
}

async fn api_analyze(Json(req): Json<AnalyzeRequest>) -> Json<AnalyzeResponse> {
    let path = PathBuf::from(&req.path);
    if path.exists() {
        match analyze_image(&path).await {
            Ok(res) => {
                let size_gb = res.file_size as f64 / (1024.0 * 1024.0 * 1024.0);
                let size_str = if size_gb >= 1.0 {
                    format!("{:.2} GB", size_gb)
                } else {
                    format!("{:.1} MB", res.file_size as f64 / (1024.0 * 1024.0))
                };
                let sha = res.hash.sha256;
                let sha_preview = if sha.len() > 12 {
                    format!("{}...{}", &sha[..6], &sha[sha.len() - 6..])
                } else {
                    sha
                };

                Json(AnalyzeResponse {
                    success: true,
                    file_path: res.file_path.to_string_lossy().to_string(),
                    file_name: res.file_name,
                    file_size_bytes: res.file_size,
                    file_size_formatted: size_str,
                    display_name: res.os_info.display_name,
                    arch: res.architecture.to_string(),
                    format: res.format.to_string(),
                    uefi_capable: res.boot_info.uefi_capable,
                    secure_boot: format!("{:?}", res.boot_info.secure_boot),
                    compat_level: res.compatibility.level.to_string(),
                    score: res.compatibility.score,
                    boot_strategy: format!("{:?}", res.boot_strategy),
                    sha256_preview: sha_preview,
                    uefi_x64_score: res.compatibility.uefi_x64_score,
                    bios_score: res.compatibility.bios_score,
                    secure_boot_score: res.compatibility.secure_boot_score,
                    arm64_score: res.compatibility.arm64_score,
                    warnings: res.warnings,
                    device_guidance: res.device_guidance,
                    error: None,
                })
            }
            Err(e) => Json(AnalyzeResponse {
                success: false,
                file_path: req.path.clone(),
                file_name: path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Unknown".to_string()),
                file_size_bytes: 0,
                file_size_formatted: "Unknown".to_string(),
                display_name: "Unknown Image".to_string(),
                arch: "x86_64".to_string(),
                format: "Unknown".to_string(),
                uefi_capable: true,
                secure_boot: "Supported".to_string(),
                compat_level: "INCOMPATIBLE".to_string(),
                score: 0,
                boot_strategy: "Unknown".to_string(),
                sha256_preview: "N/A".to_string(),
                uefi_x64_score: 0,
                bios_score: 0,
                secure_boot_score: 0,
                arm64_score: 0,
                warnings: vec![],
                device_guidance: vec![],
                error: Some(e.to_string()),
            }),
        }
    } else {
        Json(AnalyzeResponse {
            success: false,
            file_path: req.path.clone(),
            file_name: path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
            file_size_bytes: 0,
            file_size_formatted: String::new(),
            display_name: String::new(),
            arch: String::new(),
            format: String::new(),
            uefi_capable: false,
            secure_boot: String::new(),
            compat_level: String::new(),
            score: 0,
            boot_strategy: String::new(),
            sha256_preview: String::new(),
            uefi_x64_score: 0,
            bios_score: 0,
            secure_boot_score: 0,
            arm64_score: 0,
            warnings: vec![],
            device_guidance: vec![],
            error: Some(format!("File not found: {}", req.path)),
        })
    }
}
