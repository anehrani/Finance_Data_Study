# Implementation Plan - Convert bnd_ret to Rust

## Goal
Convert the C++ code in `statn/bnd_ret` to a Rust package. The package will implement a primitive moving-average-crossover system and calculate bounds on future returns using order statistics.

## User Review Required
> [!IMPORTANT]
> The original C++ code uses `conio.h` and `_getch()` which are Windows-specific and blocking. The Rust implementation will remove these blocking calls to be more CLI-friendly.

## Proposed Changes

### New Project: `bnd_ret`

#### [NEW] [Cargo.toml](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/bnd_ret/Cargo.toml)
- Define package metadata.

#### [NEW] [src/stats.rs](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/bnd_ret/src/stats.rs)
- Port `lgamma`, `ibeta`, `orderstat_tail`, `quantile_conf`.

#### [NEW] [src/main.rs](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/bnd_ret/src/main.rs)
- Implement `opt_params`, `test_system`, `main`.

#### [NEW] [README.md](file:///Users/alinehrani/projects/git_anehrani/Hilbert_project_01/statn/bnd_ret/README.md)
- Usage instructions.

## Verification Plan
- Unit tests for `stats.rs`.
- Manual run with sample data.
