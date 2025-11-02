# Development Process

**CRITICAL**: This document defines the REQUIRED development process for this project. All changes must follow this process without exception.

## Table of Contents

1. [Technology Stack Constraints](#technology-stack-constraints)
2. [Test Driven Development (TDD)](#test-driven-development-tdd)
3. [Code Quality and Maintainability](#code-quality-and-maintainability)
4. [UI Verification with MCP/Playwright](#ui-verification-with-mcpplaywright)
5. [Pre-Commit Quality Checks](#pre-commit-quality-checks)
6. [Pre-Commit Documentation Updates](#pre-commit-documentation-updates)
7. [Build and Serve Scripts](#build-and-serve-scripts)
8. [Complete Development Workflow](#complete-development-workflow)

---

## Technology Stack Constraints

### Allowed Technologies

This is a **pure Rust/Yew/WASM project**. You MUST use:

- ✅ **Rust** for all logic (domain, presentation, business logic)
- ✅ **Yew** (Rust WASM framework) for frontend
- ✅ **WASM** for browser execution
- ✅ **Cargo** for package management
- ✅ **Trunk** for WASM builds and development serving
- ✅ **basic-http-server** for serving pre-built static files
- ✅ **Bash** for simple build/deployment scripts ONLY

### Prohibited Technologies

You MUST NOT use:

- ❌ **JavaScript** (no .js files, no inline `<script>` tags, no DOM manipulation)
- ❌ **TypeScript** (no .ts files)
- ❌ **Python** (no .py files)
- ❌ **Node.js, npm, yarn, pnpm** (no JavaScript tooling)
- ❌ **Any JavaScript frameworks** (React, Vue, Angular, etc.)

### The Fundamental Rule

**If you need to add functionality, find the Rust API for it.**

If the Rust binding doesn't exist:
1. Propose creating a Rust wrapper/binding
2. Suggest an alternative Rust approach
3. Discuss with the maintainer

**NEVER fall back to JavaScript/TypeScript/Python under any circumstances.**

---

## Test Driven Development (TDD)

### Red/Green Testing Cycle

All features and fixes MUST follow the TDD cycle:

1. **RED**: Write a failing test that defines the desired behavior
   ```bash
   cargo test <test_name>  # Should fail
   ```

2. **GREEN**: Implement the minimum code to make the test pass
   ```bash
   cargo test <test_name>  # Should pass
   ```

3. **REFACTOR**: Improve code quality while keeping tests green
   ```bash
   cargo test  # All tests should still pass
   ```

### Test Coverage Requirements

- **All new features** must have tests
- **All bug fixes** must have tests that:
  - Reproduce the bug (RED)
  - Verify the fix (GREEN)
- **All public APIs** must have tests
- **All edge cases** should have tests

### Test Organization

```
crates/
  umap-core/
    src/
      module.rs         # Implementation
    tests/
      module_test.rs    # Unit tests
  umap-cli/
    src/
      main.rs          # CLI implementation
    tests/
      integration_test.rs  # Integration tests
```

### Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p umap-core

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

---

## Code Quality and Maintainability

### Modular Architecture

All code MUST be:

1. **Modular**: Organized into separate crates and modules
   - Each crate has a single, clear responsibility
   - Each module focuses on one aspect of functionality

2. **Small Functions**: Functions should be:
   - Under 50 lines when possible
   - Single responsibility
   - Well-named (intent-revealing)
   - Properly documented

3. **Documented**: All public items MUST have:
   ```rust
   /// Brief description of what this does.
   ///
   /// # Arguments
   /// * `arg` - Description of argument
   ///
   /// # Returns
   /// Description of return value
   ///
   /// # Examples
   /// ```
   /// use crate::function;
   /// assert_eq!(function(42), expected);
   /// ```
   pub fn function(arg: i32) -> i32 {
       // Implementation
   }
   ```

4. **Tested**: Every public function/module/crate MUST have tests

### Code Organization

```
crates/
  umap-core/           # Core library (no binary)
    src/
      lib.rs          # Public API
      module1.rs      # Focused module
      module2.rs      # Focused module
  umap-cli/            # CLI binary
    src/
      main.rs         # Entry point
      commands/       # Subcommands
  umap-web/            # Web UI (WASM)
    src/
      lib.rs          # Yew components
      components/     # UI components
```

---

## UI Verification with MCP/Playwright

### When UI Changes Are Made

**EVERY UI change** must be verified using MCP/Playwright:

1. Start the appropriate server:
   ```bash
   # For demo app
   bash scripts/serve-demo.sh

   # For main app
   ./target/release/umap-cli serve --db data.db --static-dir crates/umap-web/dist --addr 127.0.0.1:8080
   ```

2. Use MCP/Playwright to:
   - Navigate to the UI
   - Take screenshots
   - Verify the change appears correctly
   - Test interactions (clicks, inputs, etc.)

3. Verify across different scenarios:
   - 2D visualizations
   - 3D visualizations
   - Different datasets
   - Edge cases

### Screenshot Updates

**CRITICAL**: When the UI changes, `images/screenshot.png` MUST be updated:

1. Use MCP/Playwright to capture the latest UI:
   ```
   mcp__playwright__playwright_screenshot with appropriate parameters
   ```

2. Replace `images/screenshot.png` with the new screenshot

3. Verify the README.md shows the updated screenshot

4. **DO NOT commit UI changes without updating the screenshot**

### MCP/Playwright Workflow

```
1. Build the application
   ↓
2. Serve the application (scripts/serve-demo.sh or umap-cli serve)
   ↓
3. Navigate with playwright_navigate
   ↓
4. Take screenshot with playwright_screenshot
   ↓
5. Verify changes with playwright_get_visible_html or playwright_get_visible_text
   ↓
6. Test interactions with playwright_click, playwright_fill, etc.
   ↓
7. Update images/screenshot.png if UI changed
   ↓
8. Verify screenshot displays correctly in README
```

---

## Pre-Commit Quality Checks

Before EVERY commit, you MUST run these checks and ensure they ALL pass:

### 1. Format Check
```bash
cargo fmt --all
```
**Requirement**: No formatting changes should be needed

### 2. Build Check
```bash
cargo build --all --release
```
**Requirement**: Build must succeed with ZERO warnings

**DO NOT**:
- Add `#![allow(warnings)]` or similar directives
- Suppress warnings with `#[allow(...)]` attributes
- Ignore warnings

**DO**:
- Fix all warnings by improving the code
- Remove unused imports
- Fix unused variables
- Address all compiler suggestions

### 3. Clippy Check
```bash
cargo clippy --all --all-targets -- -D warnings
```
**Requirement**: Must pass with ZERO warnings

**DO NOT**:
- Add `#![allow(clippy::...)]` directives
- Suppress clippy warnings
- Disable lints

**DO**:
- Fix all clippy suggestions
- Improve code quality
- Follow Rust best practices
- Refactor code to satisfy clippy

### 4. Test Check
```bash
cargo test --all
```
**Requirement**: ALL tests must pass

**DO NOT**:
- Disable failing tests with `#[ignore]`
- Comment out failing tests
- Use conditional compilation to skip tests

**DO**:
- Fix the code until tests pass
- Update tests if requirements changed
- Add tests for new functionality

### 5. Demo Build Check
```bash
bash scripts/build-all.sh
```
**Requirement**: Must succeed with no errors

---

## Pre-Commit Documentation Updates

Before committing, verify and update documentation:

### 1. Check .gitignore

Ensure `.gitignore` is correct:
- All build artifacts are ignored
- All generated files are ignored (except `docs/` which is intentionally committed)
- No sensitive data is tracked

### 2. Update Relevant Documentation

For any change, consider updating:

- **CLAUDE.md**: If process, architecture, or commands change
- **README.md**: If user-facing features change
- **documentation/architecture.md**: If component structure changes
- **documentation/design.md**: If interfaces or design decisions change
- **documentation/process.md**: If development process changes
- **Code comments**: Ensure all public APIs are documented

### 3. DO NOT Delete Documentation

**NEVER**:
- Delete documentation files
- Remove sections from documentation
- Comment out documentation

**INSTEAD**:
- Update outdated documentation
- Add new sections
- Mark deprecated features clearly

---

## Build and Serve Scripts

### Always Use Scripts for Development

When developing, **ALWAYS** use `./scripts/` for building and serving:

```bash
# Build everything (workspace + demo)
bash scripts/build-all.sh

# Serve the demo locally
bash scripts/serve-demo.sh

# Test the demo
bash scripts/test-demo.sh

# Setup demo files
bash scripts/setup-demo.sh

# Create demo data
bash scripts/create-demo-data.sh
```

### Why Use Scripts?

1. **Validation**: Scripts ensure the build/serve process is correct
2. **Consistency**: Everyone uses the same process
3. **Documentation**: Scripts document the exact commands needed
4. **Maintenance**: Changes to build/serve are captured in version control

### Updating Scripts

If you need to change the build or serve process:

1. Update the appropriate script in `scripts/`
2. Test the script thoroughly
3. Update CLAUDE.md if the change affects documented commands
4. Commit the script changes with the related code changes

---

## Complete Development Workflow

### For New Features

```
1. Create a branch (optional)
   git checkout -b feature/feature-name

2. Write failing test (RED)
   - Create test file or add to existing
   - Run: cargo test <test_name>
   - Verify it fails

3. Implement feature (GREEN)
   - Write minimal code to pass test
   - Keep functions small and focused
   - Document public APIs
   - Run: cargo test <test_name>
   - Verify it passes

4. Refactor
   - Improve code quality
   - Ensure tests still pass
   - Run: cargo test --all

5. If UI changed:
   - Build: bash scripts/build-all.sh
   - Serve: bash scripts/serve-demo.sh
   - Verify with MCP/Playwright
   - Update images/screenshot.png
   - Verify README shows new screenshot

6. Pre-commit checks:
   - cargo fmt --all
   - cargo build --all --release (fix all warnings)
   - cargo clippy --all --all-targets -- -D warnings (fix all warnings)
   - cargo test --all (all tests pass)
   - bash scripts/build-all.sh (demo builds successfully)

7. Update documentation:
   - Review and update relevant .md files
   - Ensure code comments are current
   - Update CLAUDE.md if needed

8. Commit:
   - git add <files>
   - git commit -m "Descriptive message"
   - Include Co-Authored-By if working with Claude

9. Push:
   - git push origin <branch>
```

### For Bug Fixes

```
1. Reproduce the bug
   - Create a test that demonstrates the bug
   - Verify the test fails (RED)

2. Fix the bug
   - Implement the fix
   - Verify the test passes (GREEN)
   - Run all tests to ensure no regressions

3. Follow steps 4-9 from "New Features" above
```

### For UI Changes

```
1. Follow "New Features" workflow above

2. Additional UI verification:
   - Build with: bash scripts/build-all.sh
   - Serve with: bash scripts/serve-demo.sh
   - Navigate with MCP/Playwright
   - Take screenshots
   - Verify changes appear correctly
   - Test all affected visualizations (2D/3D, UMAP/PCA)

3. Update screenshot:
   - Capture new screenshot with MCP/Playwright
   - Replace images/screenshot.png
   - Verify README displays it correctly

4. Continue with pre-commit checks (step 6 from "New Features")
```

---

## Enforcement

**This process is MANDATORY and NON-NEGOTIABLE.**

Every commit MUST:
- ✅ Follow the technology stack constraints (Rust/Yew/WASM only)
- ✅ Have tests (TDD - red/green)
- ✅ Be modular and maintainable
- ✅ Have updated screenshots if UI changed
- ✅ Pass all quality checks (fmt, build, clippy, test)
- ✅ Have updated documentation
- ✅ Use build/serve scripts during development

**Violations of this process are not acceptable.**

If you're unsure about any step, ask before proceeding.
