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
where gh >nul 2>nul
if not errorlevel 1 (
    choice /m "Publish %TAG% on GitHub now with gh"
    if !errorlevel!==1 (
        gh release create %TAG% -R Timeless-XI/timeless-client --title "Timeless XI Build %BUILD%" --notes-file "%NOTES%" "%OUT%\vana-client-windows-x86_64.exe" "%OUT%\vana-client-windows-x86_64.exe.sig" "%OUT%\TimelessXI-Setup.exe"
        goto :done
    )
)
echo Upload the three files to a new release tagged %TAG%, and paste %NOTES% as its description:
start "" "https://github.com/Timeless-XI/timeless-client/releases/new?tag=%TAG%&title=Timeless%%20XI%%20Build%%20%BUILD%"
start "" "%OUT%"
:done
echo Then: python tools\client_manifest.py --latest %VER%   (add --required %VER% to make everyone update)
pause
