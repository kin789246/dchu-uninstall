@echo off
setlocal enabledelayedexpansion

:: Directory to process
set "dir_to_process=drv"

:: Command to execute
set "command=signingcheck.exe -f -p"

:: Check if directory exists
if not exist "%dir_to_process%" (
    echo The specified directory does not exist.
    exit /b 1
)

:: Iterate through each item in the directory
for /f "delims=" %%i in ('dir "%dir_to_process%" /b') do (
    set "item_path=%dir_to_process%\%%i"
    
    if exist "!item_path!\" (
        :: It's a directory
        echo Processing directory: !item_path!
        %command% "!item_path!"
    ) else (
        :: It's a file, check if it's a zip file
        if /i "%%~xi"==".zip" (
            echo Processing zip file: !item_path!
            %command% "!item_path!"
        )
    )
)

endlocal
pause