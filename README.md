# 阿里云 OSS CLI 工具 - Rust 版本

使用 Rust 重写的阿里云 OSS 命令行工具，支持文件上传、图片预览和存储桶操作。

## 功能特性

- ✅ 配置文件支持（YAML 格式，扁平/多 profile 两种格式）
- ✅ 环境变量支持
- ✅ 命令行参数支持
- ✅ 文件上传（单文件/目录）
- ✅ 图片预览（生成签名链接）
- ✅ 列出文件列表
- ✅ 生成签名访问链接
- ✅ 删除文件
- ✅ 异步高性能
- ✅ 进度条显示

## 安装

### 从源码编译

```bash
# 克隆项目
git clone https://github.com/tuyoogame/aliyun-oss-cli.rs.git
cd aliyun-oss-cli.rs

# 编译当前平台
./build.sh

# 编译所有平台（交叉编译）
./build.sh --all
```

### 预编译二进制

从 [Releases](https://github.com/tuyoogame/aliyun-oss-cli.rs/releases) 页面下载对应平台的二进制文件。

## 快速开始

### 1. 初始化配置文件

```bash
./aliyun-oss-cli init
```

这会在 `~/.aliyun-oss-cli/config.yaml` 创建配置文件。

### 2. 编辑配置文件

支持两种格式：

**扁平格式**（适合单配置场景）：

```yaml
# ~/.aliyun-oss-cli/config.yaml
endpoint: "oss-cn-hangzhou.aliyuncs.com"
access-key: "your-access-key-id"
secret-key: "your-access-key-secret"
bucket: "your-bucket-name"
```

**多 profile 格式**（适合多环境/多区域，使用 `--profile` 切换）：

```yaml
# ~/.aliyun-oss-cli/config.yaml
default:
  endpoint: "oss-cn-hangzhou.aliyuncs.com"
  access-key: "your-access-key-id"
  secret-key: "your-access-key-secret"
  bucket: "your-bucket-name"

beijing:
  endpoint: "oss-cn-beijing.aliyuncs.com"
  access-key: "your-access-key-id"
  secret-key: "your-access-key-secret"
  bucket: "your-bucket-name"
```

### 3. 使用示例

```bash
# 列出文件
./aliyun-oss-cli ls

# 列出指定前缀的文件
./aliyun-oss-cli ls images/

# 上传文件
./aliyun-oss-cli upload myimage.jpg

# 上传到指定目录
./aliyun-oss-cli upload myimage.jpg images/

# 上传目录
./aliyun-oss-cli upload myfolder/ -d

# 预览图片（生成签名链接）
./aliyun-oss-cli preview myimage.jpg

# 直接在浏览器中打开
./aliyun-oss-cli preview myimage.jpg -o

# 生成签名链接
./aliyun-oss-cli sign myimage.jpg

# 删除文件
./aliyun-oss-cli delete myimage.jpg
```

## 配置优先级

命令行参数 > 环境变量 > 配置文件

### 环境变量

```bash
export ALIYUN_OSS_ENDPOINT="oss-cn-hangzhou.aliyuncs.com"
export ALIYUN_OSS_ACCESS_KEY="your-access-key-id"
export ALIYUN_OSS_SECRET_KEY="your-access-key-secret"
export ALIYUN_OSS_BUCKET="your-bucket-name"
```

### 命令行参数（全局选项）

```bash
./aliyun-oss-cli \
  -c /path/to/config.yaml \
  --endpoint oss-cn-hangzhou.aliyuncs.com \
  --access-key your-access-key-id \
  --secret-key your-access-key-secret \
  --bucket your-bucket-name \
  ls
```

| 选项 | 短选项 | 环境变量 | 说明 |
|---|---|---|---|
| `--config` | `-c` | - | 配置文件路径 |
| `--endpoint` | - | `ALIYUN_OSS_ENDPOINT` | OSS endpoint |
| `--access-key` | - | `ALIYUN_OSS_ACCESS_KEY` | AccessKey ID |
| `--secret-key` | - | `ALIYUN_OSS_SECRET_KEY` | AccessKey Secret |
| `--bucket` | - | `ALIYUN_OSS_BUCKET` | Bucket 名称 |

## 命令说明

### init

初始化配置文件。

```bash
./aliyun-oss-cli init
```

### ls

列出 OSS 存储桶中的文件。

```bash
./aliyun-oss-cli ls [PREFIX]
./aliyun-oss-cli ls                         # 列出根目录
./aliyun-oss-cli ls images/                 # 列出 images/ 前缀的文件
./aliyun-oss-cli ls -l                      # 详细格式（显示大小和修改时间）
./aliyun-oss-cli ls -l images/              # 详细格式 + 前缀过滤
./aliyun-oss-cli ls --max-keys 50           # 单次最多列出 50 条
./aliyun-oss-cli ls --profile beijing       # 使用 beijing 配置节
```

| 选项 | 短选项 | 默认值 | 说明 |
|---|---|---|---|
| `PREFIX` | - | (root) | 位置参数，前缀过滤 |
| `--long` | `-l` | - | 详细格式 |
| `--max-keys` | - | 100 | 单次最大列出数量 |
| `--profile` | - | default | 配置文件中的配置节 |

### upload

上传文件或目录到 OSS。

```bash
./aliyun-oss-cli upload <FILE_OR_DIR> [DESTINATION]
./aliyun-oss-cli upload myimage.jpg                    # 上传文件
./aliyun-oss-cli upload myimage.jpg images/            # 上传到指定目录
./aliyun-oss-cli upload myfolder/ -d                   # 上传目录
./aliyun-oss-cli upload myimage.jpg -p images/         # 指定前缀
./aliyun-oss-cli upload myimage.jpg --public           # 公共读权限
./aliyun-oss-cli upload myimage.jpg -r                 # 覆盖已存在文件
```

| 选项 | 短选项 | 说明 |
|---|---|---|
| `DESTINATION` | - | 位置参数，目标路径 |
| `--dir` | `-d` | 上传目录模式 |
| `--prefix` | `-p` | 目标路径前缀 |
| `--public` | - | 设置为公共读 |
| `--replace` | `-r` | 覆盖已存在的文件 |
| `--profile` | - | 配置文件中的配置节 |

### preview

预览 OSS 中的图片，生成签名链接。

```bash
./aliyun-oss-cli preview <OBJECT_KEY>
./aliyun-oss-cli preview myimage.jpg              # 生成签名链接（默认 24 小时有效）
./aliyun-oss-cli preview myimage.jpg -o           # 直接在浏览器打开
./aliyun-oss-cli preview myimage.jpg --expire 3600  # 设置有效期为 1 小时
```

| 选项 | 短选项 | 默认值 | 说明 |
|---|---|---|---|
| `--open` | `-o` | - | 直接在浏览器中打开 |
| `--expire` | - | 86400 | 链接有效期（秒），默认 24 小时 |
| `--profile` | - | default | 配置文件中的配置节 |

### sign

生成文件的签名访问链接。

```bash
./aliyun-oss-cli sign <OBJECT_KEY>
./aliyun-oss-cli sign myimage.jpg                     # 下载签名链接（默认 24 小时有效）
./aliyun-oss-cli sign myimage.jpg -u                  # 上传签名链接
./aliyun-oss-cli sign myimage.jpg -e 3600             # 设置有效期为 1 小时
```

| 选项 | 短选项 | 默认值 | 说明 |
|---|---|---|---|
| `--expire` | `-e` | 86400 | 链接有效期（秒），默认 24 小时 |
| `--upload` | `-u` | - | 生成上传签名链接 |
| `--profile` | - | default | 配置文件中的配置节 |

### delete

删除 OSS 中的文件。

```bash
./aliyun-oss-cli delete <OBJECT_KEY> [OBJECT_KEY...]
./aliyun-oss-cli delete myimage.jpg                   # 删除单个文件（需确认）
./aliyun-oss-cli delete file1.jpg file2.png           # 批量删除
./aliyun-oss-cli delete myimage.jpg -q                # 静默模式，跳过确认
```

| 选项 | 短选项 | 说明 |
|---|---|---|
| `--quiet` | `-q` | 静默模式，不显示删除确认 |
| `--profile` | - | 配置文件中的配置节 |

## 构建脚本说明

### build.sh

统一构建脚本，默认编译当前平台，支持通过选项指定目标平台进行交叉编译。

```bash
# 编译当前平台（快速构建）
./build.sh

# 指定版本号，编译当前平台
./build.sh v1.0.0

# 编译所有平台（交叉编译）
./build.sh --all

# 指定版本号，编译所有平台
./build.sh --all v1.0.0

# 仅编译 Linux 所有架构
./build.sh -p linux

# 仅编译 macOS arm64
./build.sh -p darwin -a arm64

# 编译 Linux 和 Windows
./build.sh -p linux -p windows

# 编译指定平台和架构
./build.sh -p linux -a amd64
./build.sh -p windows -a amd64

# 编译所有平台但不压缩
./build.sh --all --no-compress

# 查看帮助
./build.sh --help
```

支持的平台：
- Linux: amd64, arm64, arm, 386
- Windows: amd64, arm64, 386
- macOS: amd64, arm64 (Apple Silicon)

## 技术栈

- **CLI**: clap
- **异步运行时**: tokio
- **HTTP 客户端**: reqwest
- **XML 解析**: quick-xml
- **配置管理**: serde_yaml
- **签名算法**: HMAC-SHA1
- **进度条**: indicatif

## 许可证

MIT
