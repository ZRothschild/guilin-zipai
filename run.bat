@echo off
echo =========================================
echo      桂林字牌游戏启动脚本
echo =========================================
echo.

REM 检查Rust是否安装
where rustc >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Rust未安装，请先安装Rust：
    echo    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs ^| sh
    pause
    exit /b 1
)
echo ✅ Rust已安装

REM 检查Trunk是否安装
where trunk >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ Trunk未安装，正在安装...
    cargo install trunk
    if %errorlevel% neq 0 (
        echo ❌ Trunk安装失败
        pause
        exit /b 1
    )
    echo ✅ Trunk已安装
) else (
    echo ✅ Trunk已安装
)

REM 检查WASM目标
echo 检查WASM目标...
rustup target add wasm32-unknown-unknown 2>nul
echo ✅ WASM目标已准备

echo.
echo 请选择要启动的组件：
echo 1. 服务器 (端口: 8080)
echo 2. 客户端 (端口: 3000)
echo 3. 退出
echo.

choice /c 123 /m "请选择 (1/2/3): "
if %errorlevel% equ 1 goto start_server
if %errorlevel% equ 2 goto start_client
if %errorlevel% equ 3 goto exit_script

:start_server
echo.
echo 启动游戏服务器...
echo 服务器将在 http://127.0.0.1:8080 运行
echo 按 Ctrl+C 停止服务器
cd crates\server
cargo run
goto exit_script

:start_client
echo.
echo 启动游戏客户端...
echo 客户端将在 http://127.0.0.1:3000 运行
echo 按 Ctrl+C 停止客户端
trunk serve
goto exit_script

:exit_script
echo.
echo 游戏架构：
echo   - 服务器: Rust + tokio + WebSocket (端口: 8080)
echo   - 客户端: Rust + Yew + WASM (端口: 3000)
echo.
echo 使用说明：
echo   1. 首先启动服务器
echo   2. 然后启动客户端
echo   3. 浏览器访问 http://127.0.0.1:3000
echo.
pause