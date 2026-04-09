use anyhow::Result;
use clap::Args;
use crate::config::AppConfig;
use crate::oss::OSSClient;
use crate::utils::{is_image, is_image_content_type, open_browser};

/// 预览 OSS 中的图片
#[derive(Args)]
pub struct PreviewArgs {
    /// 对象键名
    #[arg(value_name = "OBJECT_KEY")]
    object_key: String,

    /// 直接在浏览器中打开
    #[arg(short, long)]
    open: bool,

    /// 链接有效期(秒)，默认 86400 (24小时)
    #[arg(long, default_value = "86400")]
    expire: u64,

    /// 配置文件中的配置节
    #[arg(long, default_value = "default")]
    profile: String,
}

pub async fn execute(config: AppConfig, args: PreviewArgs) -> Result<()> {
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

    // 检查文件是否存在
    match client.object_exists(object_key).await {
        Ok(true) => {}
        Ok(false) => {
            anyhow::bail!("文件不存在: {}", object_key);
        }
        Err(e) => {
            anyhow::bail!("检查文件失败: {}", e);
        }
    }

    // 获取文件元信息
    let meta = client.get_object_meta(object_key).await?;
    let content_type = meta.get("content-type").map(|s| s.as_str()).unwrap_or("unknown");

    println!("📄 文件类型: {}", content_type);

    // 检查是否是图片
    if !is_image(object_key) && !is_image_content_type(content_type) {
        println!("⚠️  警告: 文件可能不是图片");
    }

    // 生成签名 URL
    let signed_url = client.generate_signed_url(object_key, args.expire)?;

    println!("\n✅ 预览链接生成成功!");
    println!("⏰ 有效期: {} 秒 (约 {:.1} 小时)", args.expire, args.expire as f64 / 3600.0);
    println!("\n🔗 预览地址:\n{}\n", signed_url);

    // 输出 Markdown 格式链接
    println!("📋 Markdown 格式:");
    println!("[{}]({})", object_key, signed_url);

    // 如果需要直接打开浏览器
    if args.open {
        println!("\n🌐 正在打开浏览器...");
        if let Err(e) = open_browser(&signed_url) {
            eprintln!("⚠️  打开浏览器失败: {}", e);
        }
    }

    Ok(())
}
