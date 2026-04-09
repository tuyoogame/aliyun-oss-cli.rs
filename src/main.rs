use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

mod cmd;
mod config;
mod oss;
mod utils;

use cmd::{delete, init, list, preview, sign, upload};
use config::AppConfig;

/// 阿里云 OSS 命令行工具
#[derive(Parser)]
#[command(name = "aliyun-oss-cli")]
#[command(about = "阿里云 OSS 命令行工具，支持文件上传、图片预览和存储桶操作")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// OSS endpoint
    #[arg(long, env = "ALIYUN_OSS_ENDPOINT")]
    endpoint: Option<String>,

    /// AccessKey ID
    #[arg(long, env = "ALIYUN_OSS_ACCESS_KEY")]
    access_key: Option<String>,

    /// AccessKey Secret
    #[arg(long, env = "ALIYUN_OSS_SECRET_KEY")]
    secret_key: Option<String>,

    /// Bucket 名称
    #[arg(long, env = "ALIYUN_OSS_BUCKET")]
    bucket: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化配置文件
    Init,
    /// 列出存储桶中的文件
    Ls(list::ListArgs),
    /// 上传文件或目录到 OSS
    Upload(upload::UploadArgs),
    /// 预览 OSS 中的图片
    Preview(preview::PreviewArgs),
    /// 生成文件的签名访问链接
    Sign(sign::SignArgs),
    /// 删除 OSS 中的文件
    Delete(delete::DeleteArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    // 加载配置
    let mut config = AppConfig::load(cli.config.as_deref())?;

    // 命令行参数覆盖配置文件
    if let Some(endpoint) = cli.endpoint {
        config.endpoint = Some(endpoint);
    }
    if let Some(access_key) = cli.access_key {
        config.access_key = Some(access_key);
    }
    if let Some(secret_key) = cli.secret_key {
        config.secret_key = Some(secret_key);
    }
    if let Some(bucket) = cli.bucket {
        config.bucket = Some(bucket);
    }

    // 如果使用了配置文件，打印提示
    if let Some(ref config_path) = config.config_file {
        info!("使用配置文件: {}", config_path.display());
    }

    // 执行命令
    match cli.command {
        Commands::Init => init::execute().await,
        Commands::Ls(args) => list::execute(config, args).await,
        Commands::Upload(args) => upload::execute(config, args).await,
        Commands::Preview(args) => preview::execute(config, args).await,
        Commands::Sign(args) => sign::execute(config, args).await,
        Commands::Delete(args) => delete::execute(config, args).await,
    }
}
