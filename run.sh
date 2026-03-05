#!/bin/bash
# 桂林字牌游戏启动脚本

echo "========================================="
echo "     桂林字牌游戏启动脚本"
echo "========================================="
echo ""

# 检查Rust和Trunk是否安装
check_dependencies() {
    echo "检查依赖..."
    
    if ! command -v rustc &> /dev/null; then
        echo "❌ Rust未安装，请先安装Rust："
        echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    echo "✅ Rust已安装"
    
    if ! command -v trunk &> /dev/null; then
        echo "❌ Trunk未安装，正在安装..."
        cargo install trunk
        if [ $? -ne 0 ]; then
            echo "❌ Trunk安装失败"
            exit 1
        fi
        echo "✅ Trunk已安装"
    else
        echo "✅ Trunk已安装"
    fi
    
    # 检查WASM目标
    echo "检查WASM目标..."
    rustup target add wasm32-unknown-unknown 2>/dev/null
    echo "✅ WASM目标已准备"
}

# 启动服务器
start_server() {
    echo ""
    echo "启动游戏服务器..."
    cd crates/server
    echo "服务器将在 http://127.0.0.1:8080 运行"
    echo "按 Ctrl+C 停止服务器"
    cargo run
}

# 启动客户端
start_client() {
    echo ""
    echo "启动游戏客户端..."
    echo "客户端将在 http://127.0.0.1:3000 运行"
    echo "按 Ctrl+C 停止客户端"
    trunk serve
}

# 显示帮助
show_help() {
    echo ""
    echo "使用方法:"
    echo "  $0 [选项]"
    echo ""
    echo "选项:"
    echo "  server     启动服务器"
    echo "  client     启动客户端"
    echo "  all        启动服务器和客户端（需要两个终端）"
    echo "  help       显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 server    # 在终端1启动服务器"
    echo "  $0 client    # 在终端2启动客户端"
    echo "  $0 all       # 提示分别启动服务器和客户端"
}

main() {
    case "$1" in
        "server")
            check_dependencies
            start_server
            ;;
        "client")
            check_dependencies
            start_client
            ;;
        "all")
            check_dependencies
            echo ""
            echo "请打开两个终端分别运行："
            echo "1. 在第一个终端运行: $0 server"
            echo "2. 在第二个终端运行: $0 client"
            echo ""
            echo "然后访问 http://127.0.0.1:3000 开始游戏"
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            echo ""
            echo "请指定要启动的组件：server、client 或 all"
            echo "使用 '$0 help' 查看帮助"
            echo ""
            echo "游戏架构:"
            echo "  - 服务器: Rust + tokio + WebSocket (端口: 8080)"
            echo "  - 客户端: Rust + Yew + WASM (端口: 3000)"
            echo ""
            echo "快速开始:"
            echo "  1. 打开终端1: $0 server"
            echo "  2. 打开终端2: $0 client"  
            echo "  3. 浏览器访问: http://127.0.0.1:3000"
            ;;
    esac
}

main "$@"