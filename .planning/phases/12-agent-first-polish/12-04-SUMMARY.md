# Plan 12-04 Summary: Cargo-sweep Integration

**Phase:** 12-agent-first-polish
**Plan:** 04
**Status:** Complete
**Duration:** ~10 min

## Objective

Integrate cargo-sweep for automatic build artifact cleanup to prevent disk saturation during development. Cancer projects can consume 10-20GB of disk space from build artifacts.

## Completed Tasks

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Add clean command module | 242a0a4 | clean.rs, mod.rs, main.rs |
| 2 | Integrate auto-sweep into serve | ad7786b | serve.rs, clean.rs |
| 3 | Add env var configuration | bd99d40 | serve.rs, env.example.tpl |

## Changes Made

### New Command: `cancer clean`

```
cancer clean [OPTIONS]

Options:
  -d, --days <DAYS>         Remove artifacts older than N days (default: 30)
  -t, --toolchains          Also remove artifacts from old toolchains
      --skip-install-check  Skip cargo-sweep installation check
```

### Auto-cleanup on `cancer serve`

- Runs automatically on every `cancer serve` startup
- Uses 7-day threshold by default (more aggressive for dev workflow)
- Silent if no artifacts cleaned or cargo-sweep not installed
- Configurable via `CARGO_SWEEP_DAYS` environment variable

### Configuration

New environment variable added to `.env.example` template:

```bash
# Build cleanup: auto-remove artifacts older than N days on `cancer serve`
# Set to 0 to disable automatic cleanup (requires cargo-sweep)
CARGO_SWEEP_DAYS=7
```

## Technical Details

### Files Modified

- `cancer-cli/src/commands/clean.rs` (new) - Clean command implementation
- `cancer-cli/src/commands/mod.rs` - Module registration
- `cancer-cli/src/commands/serve.rs` - Auto-sweep integration
- `cancer-cli/src/main.rs` - Command enum and dispatch
- `cancer-cli/src/templates/files/root/env.example.tpl` - Default configuration

### Design Decisions

1. **Silent failure by default** - Missing cargo-sweep doesn't break serve
2. **7-day default for auto-sweep** - More aggressive than manual (30 days)
3. **Environment variable override** - Set `CARGO_SWEEP_DAYS=0` to disable
4. **No auto-install** - Shows helpful message but doesn't install dependencies

## Verification

- [x] `cargo build --package cancer-cli` succeeds
- [x] `cancer clean --help` shows days and toolchains options
- [x] `cancer clean` shows install instructions when cargo-sweep missing
- [x] `cancer serve` includes sweep logic (verified via code review)
- [x] `CARGO_SWEEP_DAYS` environment variable support added
- [x] Missing cargo-sweep doesn't break serve command

## Impact

Developers no longer need to manually manage disk space from Cancer build artifacts. The framework is now self-maintaining with sensible defaults.
