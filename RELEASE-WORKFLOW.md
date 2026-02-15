# Release Workflow Documentation

## Overview

The DonutSMP Map Flipper Bot uses a **unified release workflow** that automatically builds and releases both Node.js and Rust implementations when pull requests are merged to the main branch.

## Workflow File

**Location**: `.github/workflows/unified-release.yml`

## How It Works

### Automatic Releases

When a PR is merged to `main`:
1. The workflow automatically triggers
2. Builds **both** Node.js and Rust versions
3. Creates a **single release** with **one version tag**
4. Publishes all platform binaries together

### Manual Releases

You can also trigger releases manually:
1. Go to **Actions** → **Unified Release** in GitHub
2. Click **Run workflow**
3. Specify the version tag (e.g., `v1.2.3`)
4. The workflow will build and release both implementations

## Build Process

### Stage 1: Build Node.js Version
- Installs production dependencies
- Creates archives for:
  - Windows (`.zip` with `start.bat`)
  - Linux (`.tar.gz` with `start.sh`)
  - Mac (`.tar.gz` with `start.sh`)
- Uploads artifacts for release

### Stage 2: Build Rust Version
- Builds Rust binaries using matrix strategy for:
  - Linux x86_64
  - Windows x86_64
  - macOS Intel (x86_64)
  - macOS Apple Silicon (ARM64)
- Uses Rust nightly toolchain (required for Azalea)
- Uploads binary artifacts

### Stage 3: Create Unified Release
- Downloads all artifacts from previous stages
- Creates release archives with proper naming:
  - **Node.js**: `donutsmp-mapflipper-nodejs-vX.X.X-{platform}.{ext}`
  - **Rust**: `donutsmp-mapflipper-rust-vX.X.X-{platform}-{arch}.{ext}`
- Publishes single GitHub release with all 7 platform builds
- Includes comprehensive release notes with installation instructions

## Version Tagging

**Automatic versioning** (on PR merge):
- Finds the latest tag (e.g., `v1.0.4`)
- Increments patch version (becomes `v1.0.5`)
- Uses same version for both Node.js and Rust builds

**Manual versioning** (workflow dispatch):
- You specify the exact version tag
- Both implementations use the same version

## Release Assets

Each release includes **7 files**:

### Node.js Version (3 files)
- `donutsmp-mapflipper-nodejs-vX.X.X-windows.zip`
- `donutsmp-mapflipper-nodejs-vX.X.X-linux.tar.gz`
- `donutsmp-mapflipper-nodejs-vX.X.X-mac.tar.gz`

### Rust Version (4 files)
- `donutsmp-mapflipper-rust-vX.X.X-linux-x86_64.tar.gz`
- `donutsmp-mapflipper-rust-vX.X.X-windows-x86_64.zip`
- `donutsmp-mapflipper-rust-vX.X.X-macos-x86_64.tar.gz`
- `donutsmp-mapflipper-rust-vX.X.X-macos-aarch64.tar.gz`

## Benefits of Unified Workflow

1. **Single Version Tag**: Both implementations share the same version, reducing confusion
2. **Atomic Releases**: All builds happen together or not at all
3. **Consistent Versioning**: Users always know which versions are compatible
4. **Reduced Maintenance**: One workflow to maintain instead of two
5. **Better UX**: Users find both versions in one release page

## Workflow Dependencies

```
build-nodejs ─┐
              ├─→ create-release
build-rust ───┘
```

The `create-release` job waits for both build jobs to complete before creating the release.

## Caching Strategy

To speed up builds, the workflow caches:
- **Cargo registry**: Package metadata
- **Cargo index**: Package index
- **Target directory**: Compiled artifacts

This significantly reduces Rust build times on subsequent runs.

## Permissions

The workflow requires:
- `contents: write` - To create releases and tags

This is automatically granted through `GITHUB_TOKEN`.

## Troubleshooting

### Build Failures

If a build fails:
1. Check the **Actions** tab for error logs
2. Verify all dependencies are correctly specified
3. Ensure Rust code compiles with `cargo build --release`
4. Ensure Node.js code has no missing dependencies

### Release Not Created

If builds succeed but no release appears:
1. Check that the PR was actually merged (not just closed)
2. Verify the workflow has `contents: write` permission
3. Check for conflicts with existing tags

### Wrong Version Number

If the auto-versioning produces unexpected results:
1. Use manual workflow dispatch with explicit version
2. Check existing tags: `git tag -l`
3. Ensure tags follow semver format: `vX.Y.Z`

## Migration from Old Workflows

**Previous setup** (now removed):
- `release.yml` - Node.js only
- `rust-release.yml` - Rust only

**Current setup**:
- `unified-release.yml` - Both implementations

The new workflow combines the functionality of both old workflows while ensuring consistent versioning across implementations.
