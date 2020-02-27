@echo off
cargo build --release
mkdir stream_sniper
robocopy ./target/release/ ./stream_sniper/ roblox_stream_sniper.exe
@echo Paste Cookie Here pls > ./stream_sniper/cookie
@cls
echo Executable is located in folder stream_sniper
echo First paste your cookie into the file called cookie
echo Then you can use the executable
pause