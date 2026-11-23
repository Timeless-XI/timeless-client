@echo off
setlocal
title Timeless XI client - build
cd /d "%~dp0"

rem --- Rust ---------------------------------------------------------------
where cargo >nul 2>nul
if not errorlevel 1 goto have_cargo
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where cargo >nul 2>nul
if not errorlevel 1 goto have_cargo
echo Rust is not installed yet. Install it once with this command, then run build_timeless.bat again:
echo.
echo     winget install --id Rustlang.Rustup
echo.
pause
exit /b 1
:have_cargo

rem --- 64-bit Visual Studio compiler (a plain prompt can pick the x86 libraries) ---
if defined VSCMD_ARG_TGT_ARCH goto have_vs
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if not exist "%VSWHERE%" goto no_vs
for /f "usebackq delims=" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VSINSTALL=%%i"
if not defined VSINSTALL goto no_vs
call "%VSINSTALL%\VC\Auxiliary\Build\vcvars64.bat" >nul
goto have_vs
:no_vs
echo Visual Studio's C++ tools were not found. In the Visual Studio Installer, add
echo "Desktop development with C++", then run build_timeless.bat again.
pause
exit /b 1
:have_vs

rem --- Build --------------------------------------------------------------
echo Building the Timeless XI client (the first build takes several minutes)...
cargo build --release -p vana-client
if errorlevel 1 goto failed
rem Timeless XI: the same exe under the setup name is the player download (it opens the setup wizard).
copy /y "target\release\vana-client.exe" "target\release\TimelessXI-Setup.exe" >nul
echo.
echo Built: %CD%\target\release\vana-client.exe
echo Player download (setup wizard): %CD%\target\release\TimelessXI-Setup.exe
echo Start the client with VanaLocal\Play Timeless Local.bat
pause
exit /b 0

:failed
echo.
echo BUILD FAILED. Copy the red error lines above and send them to Claude.
pause
exit /b 1
