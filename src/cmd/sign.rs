use anyhow::Result;
use clap::Args;
use chrono::Local;

use crate::config::AppConfig;
use crate::oss::OSSClient;
use crate::utils::get_content_type;

/// 生成文件的签名访问链接
#[derive(Args)]
pub struct SignArgs {
    /// 对象键名
    #[arg(value_name = "OBJECT_KEY")]
    object_key: String,

    /// 链接有效期(秒)，默认 86400 (24小时)
    #[arg(short, long, default_value = "86400")]
    expire: u64,

    /// 生成上传签名链接
    #[arg(short, long)]
    upload: bool,

    /// 配置文件中的配置节
    #[arg(long, default_value = "default")]
    profile: String,
}

pub async fn execute(config: AppConfig, args: SignArgs) -> Result<()> {
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

    let object_key = &args.object_key;

    if args.upload {
        // 生成上传签名链接
        let content_type = get_content_type(object_key);
        let signed_url = client.generate_upload_signed_url(
            object_key,
            args.expire,
            &content_type,
        )?;

        println!("📤 上传签名链接 (有效期: {}):", format_duration(args.expire));
        println!("{}", signed_url);
    } else {
        // 生成下载/预览签名链接
        let signed_url = client.generate_signed_url(object_key, args.expire)?;

        println!("📥 签名链接 (有效期: {}):", format_duration(args.expire));
        println!("{}", signed_url);
    }

    // 计算过期时间
    let expire_time = Local::now() + chrono::Duration::seconds(args.expire as i64);
    println!("⏰ 过期时间: {}", expire_time.format("%Y-%m-%d %H:%M:%S"));

    Ok(())
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}小时{}分钟", hours, minutes)
    } else if minutes > 0 {
        format!("{}分钟{}秒", minutes, secs)
    } else {
        format!("{}秒", secs)
    }
}
