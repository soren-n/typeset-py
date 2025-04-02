# Build System and Tooling

## Build System Overview

typeset-py uses a hybrid Rust-Python build system centered around Maturin for creating Python extension modules.

## Core Tools

### Maturin
**Purpose**: Build tool for PyO3-based Python extensions
**Configuration**: `pyproject.toml`

```toml
[build-system]
build-backend = "maturin"
requires = ["maturin>=1.8,<2.0"]

[tool.maturin]
features = ["pyo3/extension-module"]
```

**Key Commands**:
```bash
maturin develop    # Build and install in development mode
maturin build      # Build wheel for distribution
maturin build --release  # Optimized release build
```

### UV Package Manager
**Purpose**: Fast Python package management (preferred over pip)

```bash
uv pip install -e .              # Development install
uv pip install target/wheels/*   # Install from wheel
```

### Cargo (Rust)
**Purpose**: Rust compilation and dependency management
**Configuration**: `Cargo.toml`

```toml
[package]
name = "typeset-py"
version = "2.0.8"
edition = "2021"

[lib]
name = "typeset"
crate-type = ["cdylib"]  # Required for Python extension

[dependencies]
pyo3 = "0.25.1"
typeset = "3.1.0"
# ...
```

## Dependency Management

### Rust Dependencies

**Core Dependencies**:
- `pyo3 = "0.25.1"` - Python bindings
- `pyo3-log = "0.12.4"` - Logging bridge
- `typeset = "3.1.0"` - Core layout library
- `pest = "2.8.1"` - Parser generator
- `pest_derive = "2.8.1"` - Pest derive macros
- `lazy_static = "1.5.0"` - Global static initialization

**Dependency Updates**:
```bash
cargo update                    # Update within semver constraints
cargo search <crate>           # Check latest versions
cargo tree                     # View dependency tree
```

### Python Dependencies

**Build Requirements**:
- `maturin>=1.8,<2.0` - Build system
- `pre-commit` - Git hooks (development)
- `mypy` - Type checking (development)

**Runtime**: No Python runtime dependencies (pure extension module)

## Version Management

### Synchronized Versioning
Version must be consistent across:
- `Cargo.toml` → `version = "2.0.8"`
- `pyproject.toml` → `version = "2.0.8"`

### Version Update Process
1. Update `Cargo.toml` version
2. Update `pyproject.toml` version
3. Commit and tag release
4. CI builds and publishes automatically

## Build Targets and Platforms

### Supported Platforms (CI/CD)
- **Linux**: x86_64, aarch64, armv7, s390x, ppc64le
- **macOS**: x86_64, aarch64 (Apple Silicon)
- **Windows**: x86_64, i686

### Build Matrix
Defined in `.github/workflows/CI.yml`:
```yaml
strategy:
  matrix:
    platform:
      - runner: ubuntu-22.04
        target: x86_64
      - runner: macos-14
        target: aarch64
      # ...
```

## Development Workflow

### Local Development
```bash
# Initial setup
uv venv                    # Create virtual environment
source .venv/bin/activate  # Activate (Linux/macOS)
uv pip install pre-commit # Install development tools

# Development cycle
maturin develop           # Build and install extension
python -c "import typeset; ..."  # Test functionality
cargo test               # Run Rust tests
pre-commit run --all-files  # Run all quality checks
```

### Pre-commit Hooks
**Configuration**: `.pre-commit-config.yaml`

**Enabled Hooks**:
- **General**: trailing whitespace, end-of-file, YAML/TOML validation
- **Rust**: `cargo fmt`, `cargo clippy`, `cargo test`, `cargo audit`
- **Python**: `mypy` for type stubs
- **Documentation**: spell checking with `typos`
- **Formatting**: TOML pretty-formatting

**Installation**:
```bash
pre-commit install        # Install git hooks
pre-commit run --all-files  # Run manually
```

## Continuous Integration

### GitHub Actions Workflow
**File**: `.github/workflows/CI.yml`
**Generator**: `maturin generate-ci github` (auto-generated)

### Build Pipeline
1. **Code Quality**: Format, lint, test on multiple platforms
2. **Build Wheels**: Create platform-specific wheels
3. **Upload Artifacts**: Store wheels for download
4. **Release**: Publish to PyPI on tags (if configured)

### Test Matrix
- **Python versions**: 3.8, 3.9, 3.10, 3.11, 3.12
- **Platforms**: Linux, macOS, Windows
- **Architectures**: x86_64, aarch64, etc.

## Build Optimization

### Release Builds
```bash
maturin build --release           # Optimized binary
cargo build --release             # Direct Rust build
```

**Rust Optimization Flags** (automatically applied):
- `-C opt-level=3` - Maximum optimization
- `-C target-cpu=native` - CPU-specific optimizations (local builds)
- `strip=true` - Remove debug symbols

### Development Builds
```bash
maturin develop                   # Debug build (faster compilation)
cargo check                      # Type checking only (fastest)
```

## Troubleshooting

### Common Build Issues

**PyO3 Version Mismatch**:
```
error: PyO3 ABI version mismatch
```
Solution: Ensure consistent PyO3 versions in dependencies

**Missing Python Headers**:
```
error: Python.h not found
```
Solution: Install Python development headers (`python3-dev` on Ubuntu)

**Maturin Not Found**:
```
command not found: maturin
```
Solution: Install with `uv pip install maturin` or `pip install maturin`

**ABI Compatibility**:
```
ImportError: dynamic module does not define module export function
```
Solution: Rebuild with correct Python version/ABI

### Build Environment

**Environment Variables**:
- `PYO3_PYTHON` - Specify Python interpreter
- `CARGO_BUILD_TARGET` - Cross-compilation target
- `MACOSX_DEPLOYMENT_TARGET` - macOS compatibility

**Cross-compilation**:
```bash
# Linux to Windows
cargo install cross
cross build --target x86_64-pc-windows-gnu --release

# Using maturin
maturin build --target x86_64-pc-windows-gnu
```

## Distribution

### Wheel Building
```bash
maturin build                     # Platform-specific wheel
maturin build --find-interpreter  # Multiple Python versions
```

### Local Installation
```bash
uv pip install target/wheels/*.whl  # Install built wheel
uv pip install -e .                 # Editable development install
```

### Publishing (when configured)
```bash
maturin publish                   # Upload to PyPI
maturin publish --repository testpypi  # Test repository
```

## Performance Considerations

### Build Time Optimization
- Use `cargo check` for syntax validation
- Use `maturin develop` for development (debug builds)
- Use `cargo build --release` only for final testing

### Runtime Performance
- Release builds are ~10x faster than debug builds
- Binary size: ~2-5MB for release builds
- Startup time: <1ms (native code initialization)

### Memory Usage
- Build process: ~500MB-1GB RAM
- Runtime: Minimal overhead (native Rust performance)
- Python integration: PyO3 handles reference counting efficiently
