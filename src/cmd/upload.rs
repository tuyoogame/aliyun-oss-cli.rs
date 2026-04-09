use anyhow::{Context, Result};
use bytes::Bytes;
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use walkdir::WalkDir;

use crate::config::AppConfig;
use crate::oss::OSSClient;
use crate::utils::get_content_type;

/// 上传文件或目录到 OSS
#[derive(Args)]
pub struct UploadArgs {
    /// 本地文件或目录路径
    #[arg(value_name = "FILE_OR_DIR")]
    source: String,

    /// 目标路径
    #[arg(value_name = "DESTINATION")]
    destination: Option<String>,

    /// 目标路径前缀
    #[arg(short, long)]
    prefix: Option<String>,

    /// 上传目录
    #[arg(short, long)]
    dir: bool,

    /// 设置为公共读
    #[arg(long)]
    public: bool,

    /// 覆盖已存在的文件
    #[arg(short, long)]
    replace: bool,

    /// 配置文件中的配置节
    #[arg(long, default_value = "default")]
    profile: String,
}

pub async fn execute(config: AppConfig, args: UploadArgs) -> Result<()> {
    // 加载指定 profile 的配置
    let config = if args.profile != "default" {
        AppConfig::load_with_profile(config.config_file.as_deref(), &args.profile)?
    } else {
        config
    };

    config.validate()?;

    let client = OSSClient::new(
        config.endpoint.clone().unwrap(),
        config.access_key.clone().unwrap(),
        config.secret_key.clone().unwrap(),
    )?
    .with_bucket(config.bucket.clone().unwrap_or_default());

    let source_path = Path::new(&args.source);

    // 检查源路径是否存在
    if !source_path.exists() {
        anyhow::bail!("路径不存在: {}", args.source);
    }

    // 确定目标前缀
    let dest_prefix = args.destination.clone()
        .or(args.prefix.clone())
        .unwrap_or_default();

    if args.dir {
        // 上传目录
        if !source_path.is_dir() {
            anyhow::bail!("{} 不是目录", args.source);
        }
        upload_directory(&client, source_path, &dest_prefix).await?;
    } else {
        // 上传单个文件
        if source_path.is_dir() {
            anyhow::bail!("请使用 --dir 选项上传目录");
        }
        upload_single_file(
            &client,
            source_path,
            &dest_prefix,
            args.replace,
            args.public,
            &config,
        ).await?;
    }

    Ok(())
}

async fn upload_single_file(
    client: &OSSClient,
    file_path: &Path,
    dest_prefix: &str,
    replace: bool,
    public: bool,
    config: &AppConfig,
) -> Result<()> {
    let file_name = file_path
        .file_name()
        .context("无法获取文件名")?
        .to_string_lossy();

    // 确定目标路径
    let dest_path = if dest_prefix.is_empty() {
        file_name.to_string()
    } else if dest_prefix.ends_with('/') {
        format!("{}{}", dest_prefix, file_name)
    } else {
        dest_prefix.to_string()
    };

    // 检查文件是否已存在
    if !replace {
        match client.object_exists(&dest_path).await {
            Ok(true) => {
                anyhow::bail!("文件已存在: {} (使用 --replace 覆盖)", dest_path);
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("⚠️  检查文件失败: {}", e);
            }
        }
    }

    // 获取文件大小用于进度条
    let file_size = tokio::fs::metadata(file_path).await?.len();
    let content_type = get_content_type(&file_name);

    // 创建进度条
    let pb = ProgressBar::new(file_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("#>-"),
    );

    // 读取文件
    let data = tokio::fs::read(file_path).await?;
    pb.finish_and_clear();

    // 上传文件
    client.upload_data(&dest_path, Bytes::from(data), &content_type).await?;

    println!("✅ 上传成功: {}", dest_path);

    // 生成访问 URL
    if public {
        let url = format!(
            "https://{}.{}/{}",
            config.bucket.as_ref().unwrap(),
            config.endpoint.as_ref().unwrap(),
            dest_path
        );
        println!("🔗 公开访问地址: {}", url);
    } else {
        // 生成签名 URL
        match client.generate_signed_url(&dest_path, 3600 * 24) {
            Ok(url) => println!("🔗 签名访问地址 (24小时有效): {}", url),
            Err(e) => eprintln!("⚠️  生成签名URL失败: {}", e),
        }
    }

    Ok(())
}

async fn upload_directory(
    client: &OSSClient,
    source_dir: &Path,
    dest_prefix: &str,
) -> Result<()> {
    let mut files = Vec::new();

    // 遍历目录
    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }

    println!("发现 {} 个文件，开始上传...", files.len());

    for file_path in &files {
        // 计算相对路径
        let rel_path = file_path.strip_prefix(source_dir)?;
        let dest_path = if dest_prefix.is_empty() {
            rel_path.to_string_lossy().to_string()
        } else {
            let prefix = if dest_prefix.ends_with('/') {
                dest_prefix.to_string()
            } else {
                format!("{}/", dest_prefix)
            };
            format!("{}{}", prefix, rel_path.to_string_lossy())
        };

        let content_type = get_content_type(file_path.to_string_lossy().as_ref());

        // 读取并上传文件
        let data = tokio::fs::read(&file_path).await?;
        client.upload_data(&dest_path, Bytes::from(data), &content_type).await?;

        println!("✅ 上传成功: {}", dest_path);
    }

    println!("\n📊 目录上传完成，共 {} 个文件", files.len());

    Ok(())
}
