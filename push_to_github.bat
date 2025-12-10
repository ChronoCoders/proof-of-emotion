@echo off
REM Push Proof of Emotion to GitHub
REM
REM Usage:
REM   1. Create a new repository on GitHub: https://github.com/new
REM   2. Copy the SSH or HTTPS URL
REM   3. Run: push_to_github.bat <your-repo-url>
REM
REM Example:
REM   push_to_github.bat git@github.com:username/proof-of-emotion.git

if "%~1"=="" (
    echo [ERROR] Repository URL required
    echo.
    echo Usage: push_to_github.bat ^<repo-url^>
    echo.
    echo Example:
    echo   push_to_github.bat git@github.com:username/proof-of-emotion.git
    echo.
    echo Create new repo at: https://github.com/new
    exit /b 1
)

set REPO_URL=%~1

echo [*] Pushing Proof of Emotion to GitHub
echo Repository: %REPO_URL%
echo.

REM Add remote (ignore error if already exists)
git remote add origin "%REPO_URL%" 2>nul
if errorlevel 1 (
    git remote set-url origin "%REPO_URL%"
)

REM Rename branch to main
git branch -M main

REM Push
echo [*] Pushing to GitHub...
git push -u origin main

if errorlevel 1 (
    echo.
    echo [FAIL] Push failed. Common issues:
    echo   - Repository doesn't exist (create at: https://github.com/new)
    echo   - SSH key not configured (run: ssh -T git@github.com)
    echo   - Using HTTPS but need authentication (use SSH or GitHub CLI)
    exit /b 1
)

echo.
echo [OK] Successfully pushed to GitHub!
echo.
echo View your repository at:
echo %REPO_URL:.git=%
echo.
echo Next steps:
echo   1. Add topics/tags on GitHub (blockchain, consensus, rust)
echo   2. Enable GitHub Actions for CI/CD
echo   3. Add repository description and README badges
