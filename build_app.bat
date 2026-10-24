@echo off
setlocal EnableExtensions
cd /d "%~dp0"
set "OPEN=0"
set "CLEAN=0"
for %%A in (%*) do (
  if /i "%%~A"=="--open" ( set "OPEN=1" ) else if /i "%%~A"=="--clean" ( set "CLEAN=1" ) else if /i "%%~A"=="--help" ( goto :usage ) else if /i "%%~A"=="-h" ( goto :usage ) else ( call :usage & exit /b 2 )
)

echo.
echo [formmcp] == checking the tools
where node >nul 2>nul || ( echo [formmcp] Node 20.19 or newer is needed: https://nodejs.org & exit /b 1 )
where cargo >nul 2>nul || ( echo [formmcp] Rust is needed: https://rustup.rs ^(choose the MSVC toolchain^) & exit /b 1 )
node -e "const [a,b]=process.versions.node.split('.').map(Number); process.exit(a>22||(a===22&&b>=12)||(a===20&&b>=19)?0:1)" || ( echo [formmcp] Node 20.19 or 22.12 or newer is needed: https://nodejs.org & exit /b 1 )
rustc -vV | findstr /c:"host: " | findstr /c:"msvc" >nul || ( echo [formmcp] Rust must use the MSVC toolchain: rustup default stable-msvc & exit /b 1 )
echo [formmcp] the Rust compile needs the Visual Studio C++ build tools ^(Desktop development with C++^): https://visualstudio.microsoft.com/visual-cpp-build-tools/

echo.
echo [formmcp] == installing the interface packages and the Rust crates
call formmcp_oc_install.bat || goto :failed

if "%CLEAN%"=="1" (
  echo.
  echo [formmcp] == cleaning earlier builds
  cargo clean || goto :failed
  if exist frontend\dist rmdir /s /q frontend\dist
)

echo.
echo [formmcp] == building the desktop app and the installers ^(msi and setup.exe^)
echo [formmcp] the first build compiles a few hundred Rust crates
call frontend\node_modules\.bin\tauri.cmd build --bundles msi,nsis || goto :failed

echo.
echo [formmcp] done. The installers are in target\release\bundle\msi and target\release\bundle\nsis
if "%OPEN%"=="1" start "" "target\release\bundle"
exit /b 0

:failed
echo.
echo [formmcp] the build stopped at the step above; fix what it says and run build_app.bat again
exit /b 1

:usage
echo build_app.bat           build formMCP's desktop app on Windows, start to finish
echo   --open                open the folder with the installers when the build is done
echo   --clean               rebuild the Rust side from nothing
exit /b 0
