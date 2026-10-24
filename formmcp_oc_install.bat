@echo off
setlocal
cd /d "%~dp0"

where node >nul 2>nul || (echo Node 20.19 or newer is required: https://nodejs.org & exit /b 1)
where cargo >nul 2>nul || (echo Rust is required: https://rustup.rs & exit /b 1)

pushd frontend
call npm install || (popd & exit /b 1)
popd
cargo fetch --locked || exit /b 1

echo.
echo formMCP is ready to run from source. Start the window with: formmcp.bat
echo The MCP server works without the window: formmcp.bat mcp
echo The family list on the command line: formmcp.bat cli apps
