use anyhow::Result;
use clap::Args;
use std::io::{self, Write};

use crate::config::AppConfig;
use crate::oss::OSSClient;

/// 删除 OSS 中的文件
#[derive(Args)]
pub struct DeleteArgs {
    /// 要删除的对象键名（可指定多个）
    #[arg(value_name = "OBJECT_KEY", required = true)]
    object_keys: Vec<String>,

    /// 静默模式，不显示确认
    #[arg(short, long)]
    quiet: bool,

    /// 配置文件中的配置节
    #[arg(long, default_value = "default")]
    profile: String,
}

pub async fn execute(config: AppConfig, args: DeleteArgs) -> Result<()> {
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

    // 确认删除
    if !args.quiet && args.object_keys.len() <= 3 {
        println!("⚠️  即将删除以下 {} 个文件:", args.object_keys.len());
        for key in &args.object_keys {
            println!("  - {}", key);
        }
        print!("\n确认删除? (y/N): ");
        io::stdout().flush()?;

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;
        let confirm = confirm.trim();

        if confirm != "y" && confirm != "Y" {
            println!("已取消");
            return Ok(());
        }
    }

    // 删除文件
    let mut success_count = 0;
    let mut failed_count = 0;

    for key in &args.object_keys {
        match client.delete_object(key).await {
            Ok(_) => {
                if !args.quiet {
                    println!("✅ 已删除: {}", key);
                }
                success_count += 1;
            }
            Err(e) => {
                eprintln!("❌ 删除失败: {} - {}", key, e);
                failed_count += 1;
            }
        }
    }

    println!("\n📊 删除完成: 成功 {}, 失败 {}", success_count, failed_count);

    Ok(())
}
