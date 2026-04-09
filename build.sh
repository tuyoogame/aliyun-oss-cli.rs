#!/usr/bin/env bash

# 统一构建脚本 - 支持当前平台快速构建和交叉编译
#
# 用法:
#   ./build.sh [选项...] [版本号]
#
# 默认编译当前平台，支持通过选项指定目标平台

set -e

# ─── 配置 ────────────────────────────────────────────────────────────────────

APP_NAME="aliyun-oss-cli"
BUILD_DIR="build"
DIST_DIR="dist"

ALL_OS_LIST="linux darwin windows"
LINUX_ARCHS="amd64 arm64 arm 386"
DARWIN_ARCHS="amd64 arm64"
WINDOWS_ARCHS="amd64 arm64 386"

get_archs_for_os() {
    case "$1" in
        linux)   echo "$LINUX_ARCHS" ;;
        darwin)  echo "$DARWIN_ARCHS" ;;
        windows) echo "$WINDOWS_ARCHS" ;;
        *)       echo "" ;;
    esac
}

get_rust_target() {
    case "$1-$2" in
        linux-amd64)    echo "x86_64-unknown-linux-gnu" ;;
        linux-arm64)    echo "aarch64-unknown-linux-gnu" ;;
        linux-arm)      echo "arm-unknown-linux-gnueabihf" ;;
        linux-386)      echo "i686-unknown-linux-gnu" ;;
        darwin-amd64)   echo "x86_64-apple-darwin" ;;
        darwin-arm64)   echo "aarch64-apple-darwin" ;;
        windows-amd64)  echo "x86_64-pc-windows-msvc" ;;
        windows-arm64)  echo "aarch64-pc-windows-msvc" ;;
        windows-386)    echo "i686-pc-windows-msvc" ;;
        *)              echo "" ;;
    esac
}

is_valid_os() {
    case "$1" in
        linux|darwin|windows) return 0 ;;
        *) return 1 ;;
    esac
}

# ─── 颜色 ────────────────────────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# ─── 帮助 ────────────────────────────────────────────────────────────────────

show_help() {
    cat <<EOF
用法: $0 [选项...] [版本号]

选项:
  -p, --platform PLATFORM  目标平台: linux|darwin|windows|all (可多次指定)
  -a, --arch ARCH          目标架构: amd64|arm64|arm|386|all (可多次指定)
  --all                    编译所有平台和架构
  --compress               压缩构建产物 (交叉编译时默认开启)
  --no-compress            不压缩构建产物
  -h, --help               显示帮助信息

说明:
  不指定 -p/--all 时，默认仅编译当前平台（快速构建模式）。
  版本号可放在任意位置，自动识别（非选项参数且非平台/架构名）。
  不指定版本号时，自动从 Git 标签获取。

示例:
  $0                              # 编译当前平台
  $0 v1.0.0                       # 指定版本，编译当前平台
  $0 -p linux                     # 编译 Linux 所有架构
  $0 -p darwin -a arm64           # 编译 macOS arm64
  $0 -p linux -p windows          # 编译 Linux 和 Windows
  $0 --all                        # 编译所有平台
  $0 --all v1.0.0                 # 指定版本，编译所有平台
  $0 --all --no-compress          # 编译所有平台，不压缩

支持的平台和架构:
  Linux:   amd64, arm64, arm, 386
  macOS:   amd64, arm64
  Windows: amd64, arm64, 386
EOF
    exit 0
}

# ─── 版本号获取 ──────────────────────────────────────────────────────────────

get_version() {
    if [ -n "$1" ]; then
        echo "$1"
        return
    fi

    if command -v git &> /dev/null && git rev-parse --git-dir > /dev/null 2>&1; then
        local tag
        tag=$(git describe --tags --abbrev=0 2>/dev/null)
        if [ -n "$tag" ]; then
            local commit_count
            commit_count=$(git rev-list --count "$tag"..HEAD 2>/dev/null || echo "0")
            if [ "$commit_count" != "0" ]; then
                local short_commit
                short_commit=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
                echo "${tag}-${commit_count}-g${short_commit}"
            else
                echo "$tag"
            fi
            return
        fi
    fi

    echo "dev"
}

# ─── 检测当前平台 ────────────────────────────────────────────────────────────

detect_current_platform() {
    CURRENT_OS=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch
    arch=$(uname -m)

    case $arch in
        x86_64)        CURRENT_ARCH="amd64" ;;
        arm64|aarch64) CURRENT_ARCH="arm64" ;;
        i386|i686)     CURRENT_ARCH="386" ;;
        armv7l)        CURRENT_ARCH="arm" ;;
        *)             CURRENT_ARCH="$arch" ;;
    esac
}

# ─── 解析参数 ────────────────────────────────────────────────────────────────

OPT_PLATFORMS=""
OPT_ARCHS=""
OPT_ALL=0
OPT_COMPRESS=""
OPT_VERSION=""

parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help)
                show_help
                ;;
            -p|--platform)
                shift
                [ -z "$1" ] && error "-p 需要指定平台名"
                OPT_PLATFORMS="$OPT_PLATFORMS $1"
                ;;
            -a|--arch)
                shift
                [ -z "$1" ] && error "-a 需要指定架构名"
                OPT_ARCHS="$OPT_ARCHS $1"
                ;;
            --all)
                OPT_ALL=1
                ;;
            --compress)
                OPT_COMPRESS="yes"
                ;;
            --no-compress)
                OPT_COMPRESS="no"
                ;;
            -*)
                error "未知选项: $1"
                ;;
            *)
                if [ -z "$OPT_VERSION" ]; then
                    OPT_VERSION="$1"
                else
                    error "多余的参数: $1"
                fi
                ;;
        esac
        shift
    done

    OPT_PLATFORMS=$(echo "$OPT_PLATFORMS" | xargs)
    OPT_ARCHS=$(echo "$OPT_ARCHS" | xargs)
}

# ─── 构建目标解析 ────────────────────────────────────────────────────────────

resolve_targets() {
    local targets=""

    if [ $OPT_ALL -eq 1 ]; then
        for os in $ALL_OS_LIST; do
            for arch in $(get_archs_for_os "$os"); do
                targets="$targets $os:$arch"
            done
        done
    elif [ -n "$OPT_PLATFORMS" ]; then
        local archs="$OPT_ARCHS"
        [ -z "$archs" ] && archs="all"

        for os in $OPT_PLATFORMS; do
            is_valid_os "$os" || error "不支持的平台: $os"
            local os_archs
            os_archs=$(get_archs_for_os "$os")

            if echo " $archs " | grep -q " all "; then
                for arch in $os_archs; do
                    targets="$targets $os:$arch"
                done
            else
                for arch in $archs; do
                    local supported=0
                    for sa in $os_archs; do
                        if [ "$sa" = "$arch" ]; then
                            supported=1
                            break
                        fi
                    done
                    if [ $supported -eq 1 ]; then
                        targets="$targets $os:$arch"
                    else
                        warn "平台 $os 不支持架构 $arch，跳过"
                    fi
                done
            fi
        done
    else
        detect_current_platform
        targets="$CURRENT_OS:$CURRENT_ARCH"
    fi

    targets=$(echo "$targets" | xargs)
    [ -z "$targets" ] && error "没有有效的构建目标"
    echo "$targets"
}

# ─── 判断是否为当前平台快速构建 ──────────────────────────────────────────────

is_single_current_build() {
    [ $OPT_ALL -eq 0 ] && [ -z "$OPT_PLATFORMS" ]
}

# ─── 构建函数 ────────────────────────────────────────────────────────────────

build_target() {
    local os=$1
    local arch=$2
    local is_local=$3

    if [ "$is_local" = "1" ]; then
        info "构建 ${CYAN}$os/$arch${NC} (当前平台)..."
        cargo build --release

        local src="target/release/${APP_NAME}"
        local output="${APP_NAME}"
        if [ "$os" = "windows" ]; then
            src="${src}.exe"
            output="${output}.exe"
        fi
        cp "$src" "$output"

        if [ -f "$output" ]; then
            local size
            size=$(du -h "$output" | cut -f1)
            info "✓ 构建成功: ${BOLD}$output${NC} ($size)"
            echo ""
            info "测试运行:"
            ./"$output" --version
        else
            error "✗ 构建失败"
        fi
    else
        local target
        target=$(get_rust_target "$os" "$arch")
        [ -z "$target" ] && error "不支持的目标平台: $os/$arch"

        info "构建 ${CYAN}$os/$arch${NC} (target: $target)..."

        if ! rustup target list --installed | grep -q "$target"; then
            warn "目标平台 $target 未安装，尝试安装..."
            rustup target add "$target" || error "安装目标平台 $target 失败"
        fi

        mkdir -p "$DIST_DIR"
        cargo build --release --target "$target"

        local output="$DIST_DIR/${APP_NAME}-${VERSION}-${os}-${arch}"
        [ "$os" = "windows" ] && output="${output}.exe"

        cp "target/$target/release/${APP_NAME}" "$output" 2>/dev/null || \
        cp "target/$target/release/${APP_NAME}.exe" "$output" 2>/dev/null || \
            error "复制构建产物失败"

        if [ -f "$output" ]; then
            local size
            size=$(du -h "$output" | cut -f1)
            info "✓ 构建成功: ${BOLD}$output${NC} ($size)"
        else
            error "✗ 构建失败: $os/$arch"
        fi
    fi
}

# ─── 压缩函数 ────────────────────────────────────────────────────────────────

compress_target() {
    local os=$1
    local arch=$2
    local file="$DIST_DIR/${APP_NAME}-${VERSION}-${os}-${arch}"
    [ "$os" = "windows" ] && file="${file}.exe"
    [ ! -f "$file" ] && return

    info "压缩 $os/$arch..."
    cd "$DIST_DIR"
    if [ "$os" = "windows" ]; then
        zip "${APP_NAME}-${VERSION}-${os}-${arch}.zip" "$(basename "$file")"
    else
        tar czf "${APP_NAME}-${VERSION}-${os}-${arch}.tar.gz" "$(basename "$file")"
    fi
    cd ..
    rm "$file"
}

# ─── 校验和 ──────────────────────────────────────────────────────────────────

create_checksums() {
    info "创建 SHA256 校验和..."
    cd "$DIST_DIR"
    if command -v sha256sum &> /dev/null; then
        sha256sum *.tar.gz *.zip > sha256sums.txt 2>/dev/null || true
    else
        shasum -a 256 *.tar.gz *.zip > sha256sums.txt 2>/dev/null || true
    fi
    cd ..
}

# ─── 依赖检查 ────────────────────────────────────────────────────────────────

check_dependencies() {
    command -v cargo &> /dev/null || error "未找到 Cargo，请先安装 Rust"
    command -v git &> /dev/null || warn "未找到 Git，无法获取 commit 信息"
}

# ─── 主函数 ──────────────────────────────────────────────────────────────────

main() {
    [ ! -f "Cargo.toml" ] && error "请在项目根目录执行此脚本"

    parse_args "$@"
    check_dependencies

    VERSION=$(get_version "$OPT_VERSION")
    COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "none")
    BUILD_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    local single_current=0
    is_single_current_build && single_current=1

    local targets
    targets=$(resolve_targets)

    # 决定是否压缩
    local do_compress=0
    if [ "$OPT_COMPRESS" = "yes" ]; then
        do_compress=1
    elif [ "$OPT_COMPRESS" = "no" ]; then
        do_compress=0
    elif [ "$single_current" = "0" ]; then
        do_compress=1
    fi

    # 交叉编译模式需要 rustup
    if [ "$single_current" = "0" ]; then
        command -v rustup &> /dev/null || error "交叉编译需要 rustup，请先安装"
    fi

    echo ""
    info "开始构建 ${BOLD}${APP_NAME}${NC}"
    info "版本: ${VERSION}"
    info "Commit: ${COMMIT}"
    info "构建时间: ${BUILD_DATE}"
    if [ "$single_current" = "1" ]; then
        detect_current_platform
        info "模式: 当前平台 (${CURRENT_OS}/${CURRENT_ARCH})"
    else
        info "模式: 交叉编译"
        info "目标: ${targets}"
        [ $do_compress -eq 1 ] && info "压缩: 是" || info "压缩: 否"
    fi
    echo ""

    # 交叉编译模式：清理并创建输出目录
    if [ "$single_current" = "0" ]; then
        rm -rf "$BUILD_DIR"
        mkdir -p "$BUILD_DIR" "$DIST_DIR"
    fi

    # 构建
    for t in $targets; do
        local os=${t%%:*}
        local arch=${t##*:}
        build_target "$os" "$arch" "$single_current"
    done

    # 压缩 + 校验和（仅交叉编译模式）
    if [ "$single_current" = "0" ] && [ $do_compress -eq 1 ]; then
        echo ""
        for t in $targets; do
            local os=${t%%:*}
            local arch=${t##*:}
            compress_target "$os" "$arch"
        done
        echo ""
        create_checksums
    fi

    # 输出总结
    if [ "$single_current" = "0" ]; then
        echo ""
        info "构建完成！输出目录: ${BOLD}$DIST_DIR${NC}"
        echo ""
        info "构建产物:"
        ls -lh "$DIST_DIR"/ 2>/dev/null | tail -n +2 | awk '{print "  " $NF " (" $5 ")"}'
        rm -rf "$BUILD_DIR"
    fi
}

main "$@"
