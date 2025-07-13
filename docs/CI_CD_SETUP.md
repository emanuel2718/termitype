# CI/CD Setup for termitype

This document explains the CI/CD setup for the termitype project and how to use it effectively.

## Overview

The termitype project uses GitHub Actions for continuous integration and deployment with two main workflows:

1. **CI Workflow** (`ci.yml`) - Runs on every push and pull request
2. **Release Workflow** (`release.yml`) - Manually triggered for creating releases

## CI Workflow (Automatic)

The CI workflow runs automatically on:

- Pushes to the `main` branch
- Pull requests to the `main` branch

### What it does:

1. **Testing & Linting**

   - Formats code with `cargo fmt`
   - Lints code with `cargo clippy`
   - Runs security audit with `cargo audit`
   - Runs all tests with `cargo test`

2. **Building**

   - Builds debug version
   - Builds release version

3. **Cross-Platform Testing**
   - Tests on Ubuntu, Windows, and macOS
   - Ensures compatibility across platforms

### Key Optimizations:

- **Swatinem's Rust Cache**: Dramatically faster builds by caching Rust dependencies
- **Optimized Environment**: Disables incremental compilation and debug info for faster CI
- **Security**: Includes cargo-audit for vulnerability scanning
- **Multi-platform**: Ensures your code works across different operating systems

## Release Workflow (Manual)

The release workflow is triggered manually through the GitHub Actions interface.

### How to create a release:

1. Go to your repository on GitHub
2. Click on "Actions" tab
3. Select "Release" workflow from the left sidebar
4. Click "Run workflow" button
5. Fill in the required information:
   - **Tag name**: Version tag (e.g., `v1.0.0`)
   - **Pre-release**: Check if this is a pre-release
   - **Publish to crates.io**: Check if you want to publish to crates.io

### What the release workflow does:

1. **Pre-release Testing**

   - Runs full test suite
   - Ensures code quality with clippy and formatting checks

2. **Multi-Platform Binary Building**

   - Builds for Linux (GNU and musl)
   - Builds for Windows (x64)
   - Builds for macOS (Intel and Apple Silicon)

3. **Archive Creation**

   - Creates compressed archives (.tar.gz for Unix, .zip for Windows)
   - Includes binary, README, and LICENSE
   - Generates SHA256 checksums

4. **GitHub Release**

   - Creates a GitHub release with the specified tag
   - Uploads all binaries and checksums
   - Auto-generates release notes

5. **Optional crates.io Publishing**
   - Publishes to crates.io if enabled
   - Requires `CARGO_REGISTRY_TOKEN` secret

## Setup Requirements

### Required Secrets

If you want to publish to crates.io, you need to add the following secret:

1. Go to repository Settings → Secrets and variables → Actions
2. Add a new repository secret:
   - **Name**: `CARGO_REGISTRY_TOKEN`
   - **Value**: Your crates.io API token

To get a crates.io API token:

1. Go to [crates.io](https://crates.io)
2. Log in with your account
3. Go to Account Settings → API Tokens
4. Create a new token with appropriate permissions

### Repository Configuration

The workflows are ready to use out of the box. No additional configuration is needed for basic functionality.

## Best Practices

### Version Management

Follow semantic versioning (semver):

- `v1.0.0` - Major release (breaking changes)
- `v1.1.0` - Minor release (new features, backward compatible)
- `v1.1.1` - Patch release (bug fixes)

### Pre-release Testing

Always test your changes locally before creating a release:

```bash
# Run the same checks as CI
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

### The `--locked` Flag (Optional)

The workflows **don't** use the `--locked` flag by default, following GitHub's official examples. However, you might want to add it if:

- You want **strict reproducibility** (same exact dependency versions)
- You're having issues with dependency updates breaking CI
- You're building for production and want to ensure exact dependency versions

To add `--locked`, you can modify the workflows:

```yaml
- name: Run tests
  run: cargo test --locked
```

**When NOT to use `--locked`:**

- You want to catch dependency issues early
- You're developing a library and want to test against latest compatible versions
- You follow the "test latest compatible" approach

### Release Process

1. **Prepare the release**:

   - Update version in `Cargo.toml`
   - Update `CHANGELOG.md` if you have one
   - Commit changes to main branch

2. **Create the release**:

   - Use the GitHub Actions interface
   - Choose appropriate version tag
   - Review generated release notes

3. **Post-release**:
   - Announce the release
   - Update documentation if needed

## Advanced Features

### Cross-Platform Compatibility

The CI workflow tests on multiple platforms to catch platform-specific issues early. This includes:

- Linux (Ubuntu)
- Windows (latest)
- macOS (latest)

### Security Scanning

The workflow includes `cargo audit` which scans for:

- Security vulnerabilities in dependencies
- Unmaintained dependencies
- Deprecated dependencies

### Caching Strategy

The workflows use Swatinem's Rust cache which:

- Caches compiled dependencies
- Shares cache between similar jobs
- Automatically handles cache invalidation
- Significantly reduces build times

## Troubleshooting

### Common Issues

1. **Build fails on one platform**:

   - Check the logs for platform-specific errors
   - Test locally on that platform if possible
   - May need platform-specific dependencies

2. **Cache issues**:

   - Caches are automatically managed
   - If needed, you can clear caches in repository settings

3. **Release fails**:
   - Check that the tag doesn't already exist
   - Ensure you have write permissions to the repository
   - Verify all tests pass

### Getting Help

If you encounter issues:

1. Check the GitHub Actions logs for detailed error messages
2. Review the workflow files for configuration issues
3. Open an issue in the repository

## Future Enhancements

### Package Manager Distribution

The current setup prepares for future distribution to package managers:

- **Homebrew**: Binary releases can be used for Homebrew formulas
- **Arch AUR**: PKGBUILD can reference GitHub releases
- **Nix**: Flake already exists, can be updated to use releases
- **APT**: Debian packages can be built from releases

### Automated Version Management

Consider adding automated version bumping:

- Use tools like `cargo-release` or `release-plz`
- Automate changelog generation
- Automatic PR creation for releases

### Enhanced Testing

Future improvements might include:

- Benchmarking in CI
- Integration tests with different terminal emulators
- Performance regression testing

## Conclusion

This CI/CD setup provides a solid foundation for the termitype project with:

- Automated testing and quality checks
- Multi-platform binary releases
- Security scanning
- Optimized build times
- Easy manual release process

The setup follows current GitHub best practices while being extensible as the project grows. The workflows avoid overly opinionated choices (like `--locked`) in favor of flexibility and compatibility with standard Rust development practices.
