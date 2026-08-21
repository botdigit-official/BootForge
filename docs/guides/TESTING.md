# BootForge — Testing Guide

## Testing Strategy

BootForge uses a multi-layered testing approach:

```
┌──────────────────────────┐
│   End-to-End Tests       │  Real hardware, full workflow
├──────────────────────────┤
│   Integration Tests      │  Multiple components together
├──────────────────────────┤
│   Unit Tests             │  Individual functions/modules
└──────────────────────────┘
```

---

## Running Tests

### Quick Commands

```bash
# All unit tests
cargo test

# All tests with output
cargo test -- --nocapture

# Specific crate
cargo test -p bootforge-core
cargo test -p bootforge-analyzers
cargo test -p bootforge-engines
cargo test -p bootforge-managers

# Specific test
cargo test -p bootforge-analyzers -- iso::test_detect_ubuntu

# Run with verbose logging
RUST_LOG=debug cargo test -- --nocapture
```

### Test Categories

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Doc tests only
cargo test --doc
```

---

## Unit Tests

### ISO Analysis Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_iso9660_magic() {
        let mut data = vec![0u8; 0x8006];
        data[0x8001..0x8006].copy_from_slice(b"CD001");
        assert!(is_iso9660(&data));
    }
    
    #[test]
    fn test_detect_wim_magic() {
        let data = b"MSWIM\x00\x00\x00";
        assert!(is_wim(data));
    }
    
    #[test]
    fn test_architecture_from_pe_header() {
        assert_eq!(arch_from_machine(0x8664), Architecture::X86_64);
        assert_eq!(arch_from_machine(0xAA64), Architecture::Arm64);
        assert_eq!(arch_from_machine(0x014C), Architecture::X86);
    }
    
    #[test]
    fn test_os_detection_ubuntu_volume_id() {
        let volume_id = "Ubuntu 24.04 LTS amd64";
        let info = detect_os_from_volume_id(volume_id);
        assert_eq!(info.family, OsFamily::Linux);
        assert_eq!(info.distribution.as_deref(), Some("Ubuntu"));
        assert_eq!(info.version.as_deref(), Some("24.04 LTS"));
    }
    
    #[test]
    fn test_compatibility_score_excellent() {
        let analysis = mock_analysis_ubuntu();
        let score = calculate_compatibility(&analysis);
        assert!(score.score >= 90);
        assert_eq!(score.level, CompatLevel::Excellent);
    }
    
    #[test]
    fn test_hash_mismatch_sets_score_zero() {
        let mut analysis = mock_analysis_ubuntu();
        analysis.hash.status = VerificationStatus::Mismatch;
        let score = calculate_compatibility(&analysis);
        assert_eq!(score.score, 0);
        assert_eq!(score.level, CompatLevel::Incompatible);
    }
}
```

### Device Detection Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_drive_detection_macos() {
        let device = StorageDevice {
            platform_path: "/dev/disk0".to_string(),
            ..mock_device()
        };
        assert!(is_system_drive(&device));
    }
    
    #[test]
    fn test_usb_drive_not_system() {
        let device = StorageDevice {
            platform_path: "/dev/disk4".to_string(),
            removable: true,
            is_system_drive: false,
            ..mock_device()
        };
        assert!(!is_system_drive(&device));
    }
    
    #[test]
    fn test_device_too_small() {
        let device = StorageDevice {
            capacity: 4 * 1024 * 1024 * 1024, // 4GB
            ..mock_device()
        };
        let isos = vec![mock_iso(6 * 1024 * 1024 * 1024)]; // 6GB ISO
        assert!(!fits_on_device(&device, &isos));
    }
}
```

### Boot Configuration Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_grub_entry_ubuntu() {
        let analysis = mock_analysis_ubuntu();
        let entry = generate_grub_entry(&analysis);
        assert!(entry.contains("menuentry"));
        assert!(entry.contains("Ubuntu"));
        assert!(entry.contains("loopback"));
        assert!(entry.contains("casper"));
    }
    
    #[test]
    fn test_grub_entry_windows() {
        let analysis = mock_analysis_windows();
        let entry = generate_grub_entry(&analysis);
        assert!(entry.contains("menuentry"));
        assert!(entry.contains("Windows"));
        assert!(entry.contains("chainloader") || entry.contains("wimboot"));
    }
    
    #[test]
    fn test_partition_layout_uefi_plus_bios() {
        let layout = generate_layout(PartitionScheme::GptUefiPlusBios, 256_000_000_000);
        assert_eq!(layout.partitions.len(), 3); // EFI + BIOS + Data
        assert_eq!(layout.partitions[0].purpose, PartitionPurpose::EfiSystem);
        assert_eq!(layout.partitions[1].purpose, PartitionPurpose::BiosGrub);
        assert_eq!(layout.partitions[2].purpose, PartitionPurpose::ImageStorage);
    }
}
```

---

## Integration Tests

Integration tests verify multiple components working together.

```bash
# Run integration tests
cargo test --test integration

# With real hardware (set env var to device path)
BOOTFORGE_TEST_DEVICE=/dev/disk99 cargo test --test integration
```

### Test Scenarios

| Test | What it Verifies |
|------|-----------------|
| `test_analyze_and_score` | ISO analysis → compatibility scoring pipeline |
| `test_grub_config_generation` | Analysis → GRUB config generation |
| `test_partition_layout` | Partition layout for various ISO combinations |
| `test_download_and_verify` | Download → hash verification pipeline |
| `test_compat_db_lookup` | ISO → compatibility database matching |

---

## Test Fixtures

### Creating Test ISOs

Use the script in `scripts/create-test-isos.sh` to generate minimal ISOs for testing:

```bash
#!/bin/bash
# Create a minimal bootable ISO for testing (requires genisoimage)

mkdir -p tests/fixtures/iso

# Minimal ISO with volume ID
genisoimage -V "Ubuntu 24.04 LTS amd64" \
    -o tests/fixtures/iso/minimal-linux.iso \
    -R -J tests/fixtures/iso-content/

# Minimal ISO with EFI directory
mkdir -p /tmp/efi-test/EFI/BOOT
touch /tmp/efi-test/EFI/BOOT/BOOTx64.EFI
genisoimage -V "Test EFI ISO" \
    -o tests/fixtures/iso/minimal-efi.iso \
    -R -J /tmp/efi-test/
```

### Test Data Location

```
tests/fixtures/
├── iso/
│   ├── minimal-linux.iso       # Tiny valid Linux-like ISO
│   ├── minimal-windows.iso     # Tiny Windows-like ISO
│   ├── minimal-efi.iso         # ISO with EFI directory
│   └── corrupt.iso             # Intentionally corrupted ISO
├── wim/
│   └── sample.wim              # Minimal WIM for header parsing
├── compat/
│   └── test-compat-db.toml     # Test compatibility database
└── hashes/
    └── known-hashes.txt        # Known good hashes for test files
```

---

## Hardware Testing

### Safety Rules

> ⚠️ **NEVER run hardware tests against system drives.**
> Always use a dedicated, expendable USB drive for testing.

### Setup

```bash
# Set the test device environment variable
export BOOTFORGE_TEST_DEVICE=/dev/disk99  # macOS
export BOOTFORGE_TEST_DEVICE=/dev/sdc     # Linux
export BOOTFORGE_TEST_DEVICE=\\.\PhysicalDrive5  # Windows

# Run hardware-dependent tests
cargo test --test hardware -- --ignored
```

### Hardware Test Scenarios

| Test | Requires | Destructive? |
|------|----------|-------------|
| Device detection | USB drive plugged in | No |
| Speed benchmark (read) | USB drive | No |
| Format + partition | USB drive | **YES** |
| Write + verify | USB drive | **YES** |
| Full boot drive creation | USB drive + ISOs | **YES** |

---

## Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html --output-dir target/coverage

# Open report
open target/coverage/tarpaulin-report.html
```

### Coverage Targets

| Module | Target | Priority |
|--------|--------|----------|
| Core types | 90%+ | HIGH |
| ISO analysis | 85%+ | HIGH |
| Format detection | 90%+ | HIGH |
| Hash verification | 95%+ | CRITICAL |
| OS detection | 80%+ | HIGH |
| GRUB config gen | 85%+ | HIGH |
| System drive protection | 100% | CRITICAL |
| Device detection | 70%+ | MEDIUM |
| Partition management | 80%+ | HIGH |

---

## Performance Testing

```bash
# Benchmark ISO analysis
cargo bench -p bootforge-analyzers

# Benchmark hash computation
cargo bench -p bootforge-analyzers -- hash

# Profile with flamegraph
cargo install flamegraph
cargo flamegraph --bench analysis_bench
```

### Performance Targets

| Operation | Target | Measured |
|-----------|--------|----------|
| ISO analysis (4GB file) | < 5s | — |
| SHA256 hash (4GB file) | < 10s | — |
| Format detection | < 100ms | — |
| OS identification | < 500ms | — |
| GRUB config generation | < 50ms | — |
