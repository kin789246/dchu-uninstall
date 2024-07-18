@echo off
REM Get the current directory
set currentDir=%cd%

REM Get the parent directory
for %%i in ("%currentDir%") do set parentDir=%%~dpi

REM Remove the trailing backslash
set parentDir=%parentDir:~0,-1%

REM Extract the parent folder name
for %%i in ("%parentDir%") do set parentFolder=%%~nxi

dir /b /s *.inf > temp.txt
for /f "tokens=*" %%a in (temp.txt) do echo %%~nxa >> %parentFolder%.txt
del temp.txt