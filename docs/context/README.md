# Context Documentation

This directory contains comprehensive technical documentation for coding agents working on the typeset-py project.

## Document Overview

### [architecture.md](architecture.md)
**Core system design and patterns**
- Two-phase compilation model (Layout → Document → String)
- PyO3 wrapper patterns and memory management
- Component relationships and data flow
- Performance characteristics and thread safety
- Extension points for new features

### [dsl-grammar.md](dsl-grammar.md)
**Complete DSL specification**
- Pest grammar definition and syntax rules
- Operator precedence and associativity
- Variable substitution and template patterns
- Parsing implementation details
- Error handling and debugging

### [build-system.md](build-system.md)
**Build tooling and dependency management**
- Maturin configuration and usage
- UV package manager integration
- Cargo dependencies and versioning
- Cross-platform builds and CI/CD
- Performance optimization flags

### [api-design.md](api-design.md)
**Python-Rust integration patterns**
- PyO3 binding conventions and error handling
- Type system integration and safety
- Memory management across language boundaries
- Function signature patterns
- Testing and debugging strategies

### [development-workflow.md](development-workflow.md)
**Quality assurance and development practices**
- Pre-commit hook configuration
- Testing strategies (unit, integration, performance)
- Code quality tools (clippy, fmt, mypy, audit)
- Debugging techniques and profiling
- Release management workflow

## Usage for Coding Agents

When working on typeset-py:

1. **Start with CLAUDE.md** for quick orientation and essential commands
2. **Reference specific context docs** based on the type of work:
   - Adding DSL features → `dsl-grammar.md`
   - Performance work → `architecture.md`
   - Build issues → `build-system.md`
   - API changes → `api-design.md`
   - Quality/testing → `development-workflow.md`

3. **Keep context docs updated** when making architectural changes

## Document Maintenance

- Update context docs when making significant changes
- Verify examples and code snippets remain accurate
- Keep dependency versions current
- Add new patterns and conventions as they emerge
