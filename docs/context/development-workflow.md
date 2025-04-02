# Development Workflow and Quality Tools

## Development Environment Setup

### Prerequisites
- **Rust**: Latest stable (2021 edition)
- **Python**: 3.8+ (3.11 recommended)
- **UV**: Fast Python package manager (preferred)
- **Git**: Version control with pre-commit hooks

### Initial Setup
```bash
# Clone and setup
git clone <repository>
cd typeset-py

# Python environment (using UV)
uv venv
source .venv/bin/activate  # Linux/macOS
# .venv\Scripts\activate   # Windows

# Install development dependencies
uv pip install pre-commit
uv pip install mypy  # For type checking

# Install pre-commit hooks
pre-commit install

# Build and install extension
maturin develop
```

## Quality Assurance Pipeline

### Pre-commit Hooks Configuration

**File**: `.pre-commit-config.yaml`

The project uses comprehensive pre-commit hooks that run automatically on every commit:

#### General File Quality
- **Trailing whitespace removal**: Cleans up line endings
- **End-of-file fixing**: Ensures files end with newlines
- **YAML/TOML validation**: Syntax checking for config files
- **Merge conflict detection**: Prevents committing conflict markers
- **Large file protection**: Prevents accidental large file commits
- **Line ending normalization**: Enforces LF line endings

#### Rust Quality Tools

**Formatting (`cargo fmt`)**:
```bash
# Manual execution
cargo fmt --all

# What it checks:
# - Code style consistency
# - Indentation and spacing
# - Import organization
```

**Linting (`cargo clippy`)**:
```bash
# Manual execution
cargo clippy --all-targets --all-features -- -D warnings

# What it catches:
# - Performance issues (vec_box, needless_borrow)
# - Code smell patterns
# - Potential bugs
# - Style violations
# - Unsafe code patterns
```

**Testing (`cargo test`)**:
```bash
# Manual execution
cargo test --all-features

# What it runs:
# - Unit tests in src/
# - Integration tests in tests/
# - Doc tests in /** comments
```

**Security Auditing (`cargo audit`)**:
```bash
# Manual execution
cargo audit

# What it checks:
# - Known security vulnerabilities in dependencies
# - Yanked crates
# - Deprecated/unmaintained dependencies
```

#### Python Quality Tools

**Type Checking (`mypy`)**:
```bash
# Manual execution
mypy typeset.pyi

# What it validates:
# - Type stub accuracy
# - Function signatures
# - Return types
# - Parameter types
```

#### Documentation Quality

**Spell Checking (`typos`)**:
```bash
# Manual execution
typos

# Configuration: _typos.toml
# - Ignores intentional "typos" in documentation examples
# - Checks README.md, CLAUDE.md, docs/**/*.md
```

**TOML Formatting**:
```bash
# Automatically formats:
# - Cargo.toml
# - pyproject.toml
# - Cargo.lock
```

## Development Workflow

### Daily Development Cycle

1. **Start Development**:
   ```bash
   # Activate environment
   source .venv/bin/activate

   # Pull latest changes
   git pull origin main

   # Update dependencies if needed
   cargo update
   ```

2. **Make Changes**:
   ```bash
   # Edit code in src/
   # Edit documentation
   # Add tests if needed
   ```

3. **Test Changes**:
   ```bash
   # Quick syntax check
   cargo check

   # Build and test
   maturin develop
   python -c "import typeset; ..."  # Test functionality

   # Run quality checks
   pre-commit run --all-files
   ```

4. **Commit Changes**:
   ```bash
   git add .
   git commit -m "feat: description"  # Hooks run automatically
   ```

### Branch Workflow

**Feature Development**:
```bash
# Create feature branch
git checkout -b feature/new-dsl-operator

# Develop with frequent commits
git commit -m "wip: add grammar rules"
git commit -m "feat: implement new operator"
git commit -m "test: add operator tests"
git commit -m "docs: update grammar documentation"

# Push and create PR
git push origin feature/new-dsl-operator
```

**Quality Gates**:
- All pre-commit hooks must pass
- CI/CD pipeline must be green
- Code review required for main branch

## Testing Strategy

### Rust Testing

**Unit Tests** (in source files):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_basic() {
        let result = parse("\"hello\"", &[]);
        assert!(result.is_ok());
    }
}
```

**Integration Tests** (tests/ directory):
```rust
// tests/integration_test.rs
use typeset_py::*;

#[test]
fn test_full_pipeline() {
    let layout = text("test".to_string());
    let doc = compile(layout);
    let result = render(doc, 2, 80);
    assert_eq!(result, "test");
}
```

**Python Integration Tests**:
```python
# Test the Python API
def test_dsl_parsing():
    layout = typeset.parse('"hello" + "world"')
    doc = typeset.compile(layout)
    result = typeset.render(doc, 2, 80)
    assert result == "hello world"
```

### Performance Testing

**Benchmarks** (future):
```rust
#[bench]
fn bench_large_layout_compilation(b: &mut Bencher) {
    let layout = create_large_layout(1000);
    b.iter(|| {
        let doc = compile(layout.clone());
        black_box(doc);
    });
}
```

**Memory Testing**:
```python
# Check for memory leaks
def test_memory_usage():
    import gc, psutil

    process = psutil.Process()
    initial_memory = process.memory_info().rss

    # Create and destroy many layouts
    for _ in range(10000):
        layout = typeset.text(f"iteration {i}")
        doc = typeset.compile(layout)
        result = typeset.render(doc, 2, 80)

    gc.collect()
    final_memory = process.memory_info().rss

    # Memory should not grow significantly
    assert final_memory - initial_memory < 10_000_000  # 10MB threshold
```

## Debugging Workflow

### Rust Debugging

**Print Debugging**:
```rust
use log::{debug, info, warn, error};

fn parse_debug(input: &str) -> Result<Layout, String> {
    debug!("Parsing input: {}", input);
    let result = parse_internal(input);
    match &result {
        Ok(layout) => info!("Parse successful: {:?}", layout),
        Err(e) => error!("Parse failed: {}", e),
    }
    result
}
```

**GDB/LLDB Debugging**:
```bash
# Build with debug symbols
cargo build

# Debug with lldb (macOS) or gdb (Linux)
lldb target/debug/typeset-py
```

### Python Debugging

**Enable Rust Logging**:
```python
import logging
import typeset

# Configure Python logging to see Rust logs
logging.basicConfig(level=logging.DEBUG)

# Rust logs will appear in Python logging output
layout = typeset.parse("invalid syntax")  # Will show debug info
```

**PDB Debugging**:
```python
import pdb; pdb.set_trace()

# Debug Python integration
layout = typeset.text("debug me")
```

## Performance Monitoring

### Build Performance

**Compilation Times**:
```bash
# Measure compilation time
time cargo build --release

# Profile compilation
cargo +nightly build -Z timings
```

**Binary Size**:
```bash
# Check final binary size
ls -lh target/release/libtypeset.dylib

# Analyze binary composition
cargo bloat --release
```

### Runtime Performance

**Profiling**:
```bash
# Install profiling tools
cargo install flamegraph

# Profile specific functions
cargo flamegraph --bin typeset-py -- benchmark_args
```

**Memory Profiling**:
```bash
# Check for memory leaks
valgrind --tool=memcheck python test_script.py
```

## Continuous Integration

### GitHub Actions Pipeline

**Triggers**:
- Push to main/master
- Pull requests
- Tagged releases
- Manual workflow dispatch

**Quality Gates**:
1. **Code Quality**: All pre-commit hooks pass
2. **Cross-platform Builds**: Linux, macOS, Windows
3. **Multi-python Testing**: Python 3.8-3.12
4. **Security Scanning**: Cargo audit passes
5. **Documentation**: Builds without warnings

### Local CI Simulation

```bash
# Run full quality pipeline locally
pre-commit run --all-files   # All quality checks
cargo test --all-features    # Full test suite
maturin build --release      # Release build
python -m pytest tests/      # Python integration tests (if added)
```

## Release Workflow

### Version Management

**Prepare Release**:
```bash
# Update version in both files
# Cargo.toml: version = "2.0.9"
# pyproject.toml: version = "2.0.9"

# Update CHANGELOG.md
# Commit version bump
git commit -m "chore: bump version to 2.0.9"

# Create and push tag
git tag v2.0.9
git push origin v2.0.9
```

**Automated Release**:
- CI builds wheels for all platforms
- Uploads to PyPI (if configured)
- Creates GitHub release with artifacts

### Hotfix Workflow

```bash
# Create hotfix branch from main
git checkout -b hotfix/critical-fix

# Make minimal fix
# Test thoroughly
# Follow same quality gates

# Merge to main and tag
git checkout main
git merge hotfix/critical-fix
git tag v2.0.9
```

## Documentation Maintenance

### Context Documents

**Update Triggers**:
- API changes → Update `api-design.md`
- Grammar changes → Update `dsl-grammar.md`
- Build changes → Update `build-system.md`
- Workflow changes → Update `development-workflow.md`

**Review Process**:
- Include documentation updates in PRs
- Keep context docs in sync with code
- Validate examples still work

### README Maintenance

- Keep examples current
- Update installation instructions
- Verify all links work
- Update performance characteristics

## Troubleshooting Common Issues

### Build Issues

**PyO3 ABI Mismatch**:
```bash
# Clear build cache
cargo clean
rm -rf target/

# Rebuild everything
maturin develop
```

**Pre-commit Hook Failures**:
```bash
# Skip hooks temporarily (for emergencies)
git commit --no-verify

# Fix and re-run
pre-commit run --all-files
```

**Dependency Conflicts**:
```bash
# Check dependency tree
cargo tree

# Update problematic dependencies
cargo update -p <specific-package>
```

### Runtime Issues

**Import Errors**:
```python
# Check extension was built correctly
import sys
print(sys.path)

# Rebuild extension
maturin develop --force
```

**Performance Issues**:
```bash
# Ensure release build
maturin build --release
maturin develop --release
```
