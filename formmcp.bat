@echo off
setlocal
cd /d "%~dp0"
if not defined FORMMCP_DEV_ROOTS for %%I in ("%~dp0..") do set "FORMMCP_DEV_ROOTS=%%~fI"
if not defined FORMMCP_LAUNCHER set "FORMMCP_LAUNCHER=%~dp0formmcp.bat"

if /I "%~1"=="mcp" goto mcp
if /I "%~1"=="cli" goto cli
if /I "%~1"=="family" goto family

where cargo >nul 2>nul || (echo Rust is required: https://rustup.rs. Then run formmcp_oc_install.bat & exit /b 1)
if not exist frontend\node_modules (echo Run formmcp_oc_install.bat first. & exit /b 1)

start "formMCP interface" /b cmd /c "cd frontend && npm run dev"
timeout /t 3 /nobreak >nul
cargo run -p formmcp -- %*
exit /b %errorlevel%

:mcp
shift
cargo build -q --release --locked -p formmcp-cli 1>&2 || exit /b 1
target\release\formmcp-cli.exe mcp %1 %2 %3 %4 %5 %6 %7 %8 %9
exit /b %errorlevel%

:cli
shift
cargo build -q --release --locked -p formmcp-cli 1>&2 || exit /b 1
set "REST="
:cli_rest
if "%~1"=="" goto cli_run
set "REST=%REST% %1"
shift
goto cli_rest
:cli_run
target\release\formmcp-cli.exe%REST%
exit /b %errorlevel%

:family
shift
set "REST="
:family_rest
if "%~1"=="" goto family_run
set "REST=%REST% %1"
shift
goto family_rest
:family_run
python setup\family.py%REST%
exit /b %errorlevel%
