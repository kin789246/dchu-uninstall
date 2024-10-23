@echo off

mkdir publish
copy /y target\debug\dchu-uninstall.exe publish\
copy /y how-to-use.txt publish\
copy /y releasenote.txt publish\
copy /y gen_inf_list.cmd publish\
copy /y uninstall.cmd publish\

pause