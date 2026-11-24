@echo off
setlocal enabledelayedexpansion
rem Timeless XI: makes a client release from the Windows build, signed with the owner's key, ready to publish.
rem   1. source\build_timeless.bat     (builds target\release\vana-client.exe)
rem   2. source\release\sign_release.bat   (this: the three release files in target\release-out)
rem   3. publish them as release v<version> on Timeless-XI/timeless-client (with gh if installed, else the web page),
rem      with release\notes\v<version>.md as its notes: players see them in game ("What's New") after updating
rem   4. tools\client_manifest.py --latest <version> [--required <version>], commit, push main
rem Key: TIMELESS_RELEASE_KEY, else C:\FFXILocal\keys\timeless-release.pem (never in the repository).
cd /d "%~dp0.."
set "KEY=%TIMELESS_RELEASE_KEY%"
if "%KEY%"=="" set "KEY=C:\FFXILocal\keys\timeless-release.pem"
if not exist "%KEY%" ( echo No release key at %KEY% & pause & exit /b 1 )
for /f "tokens=3 delims= " %%v in ('findstr /b /c:"version = " crates\vana-client\Cargo.toml') do if not defined VER set "VER=%%~v"
if "%VER%"=="" ( echo Could not read the version from Cargo.toml & pause & exit /b 1 )
set "TAG=v%VER%"
set "BUILD=%VER:*-timeless.=%"
if not exist "target\release\vana-client.exe" ( echo Build first: build_timeless.bat & pause & exit /b 1 )
set "NOTES=release\notes\%TAG%.md"
if not exist "%NOTES%" ( echo Write the players' release notes first: %NOTES% & pause & exit /b 1 )
set "OUT=target\release-out\%TAG%"
if not exist "%OUT%" mkdir "%OUT%"
copy /y "target\release\vana-client.exe" "%OUT%\vana-client-windows-x86_64.exe" >nul
copy /y "target\release\vana-client.exe" "%OUT%\TimelessXI-Setup.exe" >nul
"%OUT%\vana-client-windows-x86_64.exe" release-sign --key "%KEY%" --tag %TAG% "%OUT%\vana-client-windows-x86_64.exe"
if errorlevel 1 ( echo Signing failed. & pause & exit /b 1 )
echo.
echo Release %TAG% is ready in %CD%\%OUT%:
dir /b "%OUT%"
echo.
rem GitHub's command-line tool: on PATH, else the copy in C:\FFXILocal\tools\gh.
set "GH=gh"
where gh >nul 2>nul
if errorlevel 1 set "GH=C:\FFXILocal\tools\gh\bin\gh.exe"
if /i not "%GH%"=="gh" if not exist "%GH%" goto :manual
"%GH%" auth status >nul 2>nul
if errorlevel 1 (
    echo Sign in to GitHub once so releases can be published from this computer.
    "%GH%" auth login --hostname github.com --git-protocol https --web
    "%GH%" auth status >nul 2>nul
    if errorlevel 1 goto :manual
)
echo Publishing %TAG% on GitHub ...
"%GH%" release create %TAG% -R Timeless-XI/timeless-client --target timeless-release --title "Timeless XI Build %BUILD%" --notes-file "%NOTES%" "%OUT%\vana-client-windows-x86_64.exe" "%OUT%\vana-client-windows-x86_64.exe.sig" "%OUT%\TimelessXI-Setup.exe"
if errorlevel 1 ( echo Publishing failed: see the message above. & goto :manual )
echo.
echo Published: https://github.com/Timeless-XI/timeless-client/releases/tag/%TAG%
echo Player download: https://github.com/Timeless-XI/timeless-client/releases/latest/download/TimelessXI-Setup.exe
goto :done
:manual
echo Upload the three files to a new release tagged %TAG%, and paste %NOTES% as its description:
start "" "https://github.com/Timeless-XI/timeless-client/releases/new?tag=%TAG%&title=Timeless%%20XI%%20Build%%20%BUILD%"
start "" "%OUT%"
:done
echo Then: python tools\client_manifest.py --latest %VER%   (add --required %VER% to make everyone update)
pause
