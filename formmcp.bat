@echo off
setlocal
cd /d "%~dp0"

if /I "%~1"=="mcp" (
  cargo build -q --release --locked -p formmcp-cli 1>&2 || exit /b 1
  target\release\formmcp-cli.exe mcp
  exit /b %errorlevel%
)
if /I "%~1"=="cli" (
  shift
  cargo build -q --release --locked -p formmcp-cli 1>&2 || exit /b 1
  target\release\formmcp-cli.exe %1 %2 %3 %4 %5 %6 %7 %8 %9
  exit /b %errorlevel%
)
if /I "%~1"=="family" (
  shift
  python setup\family.py %1 %2 %3 %4 %5 %6 %7 %8 %9
  exit /b %errorlevel%
)

where cargo >nul 2>nul || (echo Rust is required: https://rustup.rs. Then run formmcp_oc_install.bat & exit /b 1)
if not exist frontend\node_modules (echo Run formmcp_oc_install.bat first. & exit /b 1)

start "formMCP interface" /b cmd /c "cd frontend && npm run dev"
timeout /t 3 /nobreak >nul
cargo run -p formmcp -- %*
