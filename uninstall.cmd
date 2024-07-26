@echo off

REM change language to en_US
chcp 437

pushd %~dp0

for %%i in ("%cd%") do set fileName=%%~nxi.txt
if exist %fileName% del %fileName%
dir /b /s *.inf > temp.txt
for /f "tokens=*" %%a in (temp.txt) do echo %%~nxa >> raw.txt
del temp.txt

set "prev="
for /f "delims=" %%F in ('sort raw.txt') do (
  set "curr=%%F"
  setlocal enabledelayedexpansion
  if "!prev!" neq "!curr!" echo !curr!
  endlocal
  set "prev=%%F"
) >>%fileName%
del raw.txt

dchu-uninstall.exe -s -f %fileName%

popd

pause