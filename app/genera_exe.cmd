@echo off
:: Forzar vinculacion estatica para que no pida VCRUNTIME140.dll
set RUSTFLAGS=-C target-feature=+crt-static
cargo build --release
if not exist release mkdir release
move /y target\release\comic.exe "release\comic order.exe"
echo.
echo === Generado: release\comic order.exe (Independiente) ===
