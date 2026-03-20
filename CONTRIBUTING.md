# Contributing to Primarch.sh

Thank you for your interest in contributing to Primarch.sh! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions. We're building something together.

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/primarch-sh/terminal.git
cd terminal

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

## Project Structure

- `src/` - Vue 3 frontend (TypeScript)
- `src-tauri/` - Rust backend (Tauri)
- `src-tauri/src/pty/` - Terminal/PTY management
- `src-tauri/src/git/` - Git integration
- `src-tauri/src/workspace/` - Workspace persistence

## Development Workflow

### Branching

- Create feature branches from `main`
- Use descriptive branch names: `feature/add-theme-support`, `fix/terminal-resize-bug`

### Code Style

#### TypeScript/Vue

- Use TypeScript for all new code
- Follow Vue 3 Composition API patterns
- Use Pinia for state management
- Format with Prettier (if configured)

#### Rust

- Follow standard Rust conventions
- Run `cargo clippy` before committing
- Run `cargo fmt` to format code

### Commits

Write clear, descriptive commit messages:

```
feat: add branch deletion confirmation dialog

- Added modal confirmation before deleting branches
- Prevents accidental deletion of important branches
```

Use conventional commit prefixes:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Adding tests
- `chore:` - Maintenance tasks

## Submitting Changes

### Pull Requests

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting:
   ```bash
   # TypeScript check
   npx vue-tsc --noEmit

   # Rust checks
   cd src-tauri
   cargo clippy
   cargo fmt --check
   ```
5. Push to your fork
6. Open a Pull Request against `main`

### PR Guidelines

- Provide a clear description of the changes
- Reference any related issues
- Include screenshots for UI changes
- Keep PRs focused - one feature/fix per PR

## Reporting Issues

### Bug Reports

Include:
- Steps to reproduce
- Expected behavior
- Actual behavior
- System information (Windows version, etc.)
- Screenshots if applicable

### Feature Requests

Include:
- Clear description of the feature
- Use case / why it would be useful
- Any implementation ideas (optional)

## Architecture Notes

### Frontend (Vue)

- Components are organized by feature (`terminal/`, `git/`, `layout/`)
- State is managed through Pinia stores
- IPC with Rust backend uses `@tauri-apps/api`

### Backend (Rust)

- PTY sessions are managed by `PtyManager`
- Git operations use `git2` crate
- All Tauri commands are defined in `lib.rs`

### Adding a New Tauri Command

1. Implement the function in the appropriate module
2. Add the wrapper in `lib.rs` with `#[tauri::command]`
3. Register in `invoke_handler!`
4. Add TypeScript types in `src/types/index.ts`
5. Call from frontend using `invoke()`

## Testing

Currently, the project needs test infrastructure. If you're interested in helping set this up, that would be a valuable contribution!

## Questions?

Feel free to open an issue for questions or discussions.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
