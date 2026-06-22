@echo off
rem git-heatmap hook wrapper (Windows cmd)
rem - Always exit 0 (harness requirement)
rem - Resolve binary relative to this script's parent
rem - Swallow stdin JSON

setlocal EnableDelayedExpansion

if defined CLAUDE_PLUGIN_ROOT (
  set "PLUGIN_ROOT=!CLAUDE_PLUGIN_ROOT!"
) else (
  set "PLUGIN_ROOT=%~dp0.."
)

if defined CLAUDE_PROJECT_DIR (
  set "PROJECT_DIR=!CLAUDE_PROJECT_DIR!"
) else (
  set "PROJECT_DIR=%CD%"
)

rem Swallow stdin (avoid pipe blocking on Windows)
more < nul > nul 2>&1

set "BIN=%PLUGIN_ROOT%\bin\git-heatmap.exe"

if not exist "%BIN%" (
  echo {"systemMessage":"git-heatmap: binary not found at %BIN%. Run /heatmap once or build per README."}
  exit /b 0
)

where git >nul 2>&1
if errorlevel 1 (
  echo {"systemMessage":"git-heatmap: git not found on PATH. Install git and retry."}
  exit /b 0
)

rem Run binary; capture stdout+stderr; never let exit code fail the hook.
pushd "%PROJECT_DIR%" >nul 2>&1
"%BIN%" %* > "%TEMP%\git-heatmap-out.txt" 2>&1
set "RC=!errorlevel!"
popd >nul 2>&1

if not "!RC!"=="0" (
  echo {"systemMessage":"git-heatmap: binary exited !RC!. See %TEMP%\git-heatmap-out.txt"}
  exit /b 0
)

echo {"systemMessage":"git-heatmap: charts refreshed at .git\heatmap\. See %TEMP%\git-heatmap-out.txt for stdout."}
exit /b 0