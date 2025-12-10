@echo off
setlocal enabledelayedexpansion

echo.
echo ======================================
echo   Proof of Emotion - Test Suite
echo ======================================
echo.

REM Store original error level
set "FAILED=0"

echo [1/6] Checking code compilation...
cargo check --quiet
if errorlevel 1 (
    echo    [FAIL] Code check failed
    set "FAILED=1"
    goto end
)
echo    [PASS]

echo.
echo [2/6] Running clippy linter...
cargo clippy --quiet -- -D warnings
if errorlevel 1 (
    echo    [FAIL] Clippy found warnings
    set "FAILED=1"
    goto end
)
echo    [PASS]

echo.
echo [3/6] Running unit tests (32 tests)...
cargo test --lib --quiet
if errorlevel 1 (
    echo    [FAIL] Unit tests failed
    set "FAILED=1"
    goto end
)
echo    [PASS]

echo.
echo [4/6] Running integration tests (11 tests)...
cargo test --test integration_tests --quiet
if errorlevel 1 (
    echo    [FAIL] Integration tests failed
    set "FAILED=1"
    goto end
)
echo    [PASS]

echo.
echo [5/6] Running doctest (1 test)...
cargo test --doc --quiet
if errorlevel 1 (
    echo    [FAIL] Doctest failed
    set "FAILED=1"
    goto end
)
echo    [PASS]

echo.
echo [6/6] Running example - staking rewards...
cargo run --example staking_rewards_ascii --quiet
if errorlevel 1 (
    echo    [FAIL] Staking example failed
    set "FAILED=1"
    goto end
)
echo    [PASS]

:end
echo.
echo ======================================
if "%FAILED%"=="1" (
    echo   TEST SUITE FAILED
    echo ======================================
    exit /b 1
) else (
    echo   ALL TESTS PASSED
    echo ======================================
    echo.
    echo Test Summary
    echo   - Unit tests - 32 passed
    echo   - Integration - 11 passed
    echo   - Doctests - 1 passed
    echo   - Examples - Staking demo passed
    echo   - Total - 44 tests passed
    echo.
    echo Additional examples (ASCII output for Windows)
    echo   cargo run --example basic_consensus_ascii
    echo   cargo run --example staking_rewards_ascii
    echo.
    echo Or run with emojis (may display incorrectly)
    echo   cargo run --example basic_consensus
    echo   cargo run --example multi_validator
    echo.
)

endlocal
