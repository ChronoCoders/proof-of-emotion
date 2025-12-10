@echo off
REM Build and test script for Proof of Emotion (Windows)

echo.
echo Building Proof of Emotion Consensus
echo =======================================
echo.

REM Check Rust installation
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Cargo not found. Please install Rust from https://rustup.rs/
    exit /b 1
)

echo Rust toolchain found
rustc --version
cargo --version
echo.

REM Build
echo Building project...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Build failed
    exit /b 1
)
echo Build successful
echo.

REM Run tests
echo Running tests...
cargo test --all
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Tests failed
    exit /b 1
)
echo All tests passed
echo.

REM Build examples
echo Building examples...
cargo build --examples
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Examples build failed
    exit /b 1
)
echo Examples built
echo.

echo.
echo All checks passed!
echo.
echo To run examples:
echo   cargo run --example basic_consensus
echo   cargo run --example multi_validator
echo   cargo run --example staking_rewards
