use anyhow::Result;
use clap::Args;

use crate::config::AppConfig;
use crate::oss::OSSClient;
use crate::utils::format_size;

/// 列出存储桶中的文件
#[derive(Args)]
pub struct ListArgs {
    /// 前缀过滤（可直接作为位置参数传入，如 ls sdk-package/）
    #[arg(value_name = "PREFIX")]
    prefix: Option<String>,

    /// 单次最大列出数量
    #[arg(long, default_value = "100")]
    max_keys: i32,

    /// 配置文件中的配置节
    #[arg(long, default_value = "default")]
    profile: String,

    /// 详细格式
    #[arg(short, long)]
    long: bool,
}

pub async fn execute(config: AppConfig, args: ListArgs) -> Result<()> {
    // 加载指定 profile 的配置
    let config = if args.profile != "default" {
        AppConfig::load_with_profile(config.config_file.as_deref(), &args.profile)?
    } else {
        config
    };

    config.validate()?;

    let client = OSSClient::new(
        config.endpoint.unwrap(),
        config.access_key.unwrap(),
        config.secret_key.unwrap(),
    )?
    .with_bucket(config.bucket.unwrap_or_default());

    let prefix = args.prefix.as_deref();

    println!("\n📁 Bucket: {}", client.get_bucket()?);
    println!("📂 Prefix: {}\n", prefix.unwrap_or("(root)"));

    if args.long {
        println!("{:<40} {:>10} {:>20}", "文件名", "大小", "最后修改");
        println!("{}", "-".repeat(75));
    }

    let mut total_size: i64 = 0;
    let mut file_count = 0;
    let mut marker: Option<String> = None;

    loop {
        let (objects, prefixes, next_marker) = client
            .list_objects(prefix, args.max_keys, marker.as_deref())
            .await?;

        // 打印文件
        for obj in &objects {
            if args.long {
                println!(
                    "{:<40} {:>10} {:>20}",
                    obj.key,
                    format_size(obj.size),
                    &obj.last_modified[..19].replace('T', " ")
                );
            } else {
                println!("{}", obj.key);
            }
            total_size += obj.size;
            file_count += 1;
        }

        // 打印目录前缀
        for prefix in &prefixes {
            if args.long {
                println!("{:<40} {:>10} {:>20}", format!("{}/", prefix), "-", "-");
            } else {
                println!("{}/", prefix);
            }
        }

        if next_marker.is_none() || next_marker.as_ref() == marker.as_ref() {
            break;
        }
        marker = next_marker;
    }

    println!("\n共 {} 个文件, 总大小: {}", file_count, format_size(total_size));

    Ok(())
}
