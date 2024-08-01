:: uninstall command v1.02 by kin|jiaching

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

:: find DSP device on RPL MTL series CPU and remove then re-scan for Intel SST OED

set intcaudio=INTELAUDIO
set "pscmd=powershell -command "get-pnpdevice ^| where-object { $_.name -like '*High Definition DSP*' } ^| select-object -property instanceid""
for /f "tokens=1 delims=" %%i in ( '%pscmd% ^| findstr %intcaudio%' ) do (
  set dsp="%%i"
)

if defined dsp ( 
  echo remove device %dsp%
  pnputil /remove-device %dsp% /subtree
  pnputil /scan-devices
)

popd

pause