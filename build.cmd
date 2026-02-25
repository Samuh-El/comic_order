@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat" x64 >nul 2>&1
set PATH=%USERPROFILE%\.cargo\bin;%PATH%
cd /d "c:\Users\Samuel\Downloads\comic"
cargo run
