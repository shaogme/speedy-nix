# Speedy Nix

A GitHub Action and Mirror for ultra-fast Nix setup in CI.

This repository serves two purposes:
1. **Mirror**: Automatically mirrors the latest stable Nix version (x64/ARM64 Linux & macOS) into GitHub Releases.
2. **Action**: Provides a composite GitHub Action to download and install this pre-packaged Nix setup in seconds.

## Usage

Use this action in your workflow to install Nix significantly faster than the official installer, as it downloads a pre-built archive from GitHub Releases.

### Basic Usage

```yaml
steps:
  - uses: actions/checkout@v6
  
  - name: Install Nix
    uses: shaogme/speedy-nix@main
    # No configuration needed for standard usage
      
  - run: nix --version
```

### With Channel Configuration

To use `nix-shell` and other commands that require `<nixpkgs>`, you can configure a channel during setup:

```yaml
steps:
  - uses: actions/checkout@v6
  
  - name: Install Nix with Unstable Channel
    uses: shaogme/speedy-nix@main
    with:
      channel: 'https://nixos.org/channels/nixpkgs-unstable nixpkgs'
      
  - run: nix-shell -p hello --run hello
```

### Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `repository` | The repository hosting the releases. Change this if you are forking. | `shaogme/speedy-nix` |
| `channel` | Nix channel to add (e.g. `https://nixos.org/channels/nixpkgs-unstable nixpkgs`). | *None* |

> **Note**: This action always installs the **latest** stable version of Nix available in the mirror. Pinning specific versions is not supported to ensure users are always on the latest stable release.

## Supported Platforms

- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)

## How it works

1. A scheduled workflow (`mirrors-release.yml`) runs every 6 hours.
2. It uses a custom Rust tool (`version-check`) to query the official Nix repository for the latest stable tags.
3. If a new version is detected (compared to the state in `state.json` release asset), it:
    - Downloads the official artifacts for all supported platforms.
    - Repackages them into `.tar.zst` for optimal size and decompression speed.
    - Uploads them to a new GitHub Release.
4. The `action.yml` in this repo downloads the appropriate archive for the runner's OS/Arch, extracts it, and configures the environment.

## License

MIT / Apache-2.0
