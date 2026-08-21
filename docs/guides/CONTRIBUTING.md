# BootForge — Contributing Guide

Thank you for your interest in contributing to BootForge! This guide will help you get started.

---

## How to Contribute

### 1. Report Bugs

Found a bug? Open an issue with:
- Clear description of the problem
- Steps to reproduce
- Expected behavior vs actual behavior
- Your OS, BootForge version, and USB device info
- ISO file that caused the issue (name and source, not the file itself)

### 2. Suggest Features

Have an idea? Open a feature request with:
- Description of the feature
- Why it would be useful
- How you envision it working
- Any technical considerations

### 3. Submit Code

#### Setup

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Follow the [Development Guide](DEVELOPMENT.md) for setup
4. Make your changes
5. Write tests for new functionality
6. Ensure all tests pass: `cargo test`
7. Ensure code is formatted: `cargo fmt`
8. Ensure no lint warnings: `cargo clippy -- -D warnings`
9. Commit with a clear message
10. Open a Pull Request

#### Code Standards

- **Rust**: Follow standard Rust style, use `cargo fmt` and `clippy`
- **JavaScript**: Clean, readable code with JSDoc comments
- **CSS**: Use CSS custom properties, BEM naming
- **Documentation**: Update relevant docs for any public API changes
- **Tests**: Add unit tests for new functions, integration tests for new features
- **Commits**: Use conventional commit messages (feat/fix/docs/test/refactor/chore)

### 4. Contribute to Compatibility Database

Tested an ISO that's not in our database? Submit a compatibility entry:

1. Test the ISO with BootForge
2. Document the results in TOML format (see [COMPATIBILITY_DB.md](../specs/COMPATIBILITY_DB.md))
3. Submit as a Pull Request to `config/compat/`

### 5. Improve Documentation

- Fix typos or unclear wording
- Add examples or diagrams
- Translate documentation
- Improve README

---

## Code of Conduct

- Be respectful and constructive
- Welcome newcomers and help them learn
- Focus on the technical merits of contributions
- Disagree respectfully, with reasoning

---

## Development Areas

### Beginner-Friendly

| Area | Skills Needed |
|------|---------------|
| Documentation | Technical writing |
| Compatibility DB entries | Testing ISOs |
| UI improvements | HTML/CSS/JS |
| Bug fixes | Rust (basics) |

### Intermediate

| Area | Skills Needed |
|------|---------------|
| ISO analysis improvements | Rust, binary formats |
| New OS detection | Rust, Linux knowledge |
| UI components | JS, CSS |
| Test coverage | Rust testing |

### Advanced

| Area | Skills Needed |
|------|---------------|
| Boot engine improvements | GRUB, UEFI, bootloader knowledge |
| Platform-specific code | macOS/Windows/Linux system APIs |
| Partition management | Block device operations, filesystem internals |
| BootForge OS development | Linux distro building, kernel configuration |

---

## Architecture Decisions

Before making significant architecture changes, please:
1. Open a discussion issue describing the change
2. Reference relevant specs in the `docs/` directory
3. Get maintainer feedback before large implementations
4. Update documentation to reflect the changes

---

## Recognition

All contributors are recognized in:
- `CONTRIBUTORS.md` file
- Release notes for the version their contribution ships in
- Project website (for significant contributions)

Thank you for helping make BootForge better!
