@echo off
REM start_services.bat — Start all IntentKernel v1 daemons on Windows
REM
REM Usage:
REM   cd platform
REM   start_services.bat          start all three daemons
REM   start_services.bat --stop   stop all running daemons (requires saved PIDs)

setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "PID_DIR=%USERPROFILE%\.intentos\pids"

if not exist "%PID_DIR%" mkdir "%PID_DIR%"

if "%1"=="--stop" goto :stop_all

:start_all
echo.
echo   Starting IntentKernel services ...
echo   --------------------------------------------------
echo.

echo   Starting intentd on http://127.0.0.1:5001 ...
start "intentd" /MIN cmd /c "cd /d %SCRIPT_DIR% && python -m intentd > %USERPROFILE%\.intentos\intentd.log 2>&1"

echo   Starting capd on http://127.0.0.1:5002 ...
start "capd" /MIN cmd /c "cd /d %SCRIPT_DIR% && python -m capd > %USERPROFILE%\.intentos\capd.log 2>&1"

echo   Starting ip-descramblerd on http://127.0.0.1:5003 ...
start "ip-descramblerd" /MIN cmd /c "cd /d %SCRIPT_DIR% && python -m ip_descramblerd > %USERPROFILE%\.intentos\ip_descramblerd.log 2>&1"

echo.
echo   Services started:
echo     intentd          ^> http://127.0.0.1:5001
echo     capd             ^> http://127.0.0.1:5002
echo     ip-descramblerd  ^> http://127.0.0.1:5003
echo.
echo   Demo:
echo     python demo\secure_curl.py http://example.com --verbose
echo.
goto :eof

:stop_all
echo.
echo   Stopping IntentKernel services ...
taskkill /FI "WINDOWTITLE eq intentd" /F 2>nul
taskkill /FI "WINDOWTITLE eq capd" /F 2>nul
taskkill /FI "WINDOWTITLE eq ip-descramblerd" /F 2>nul
echo   Done.
echo.
goto :eof
