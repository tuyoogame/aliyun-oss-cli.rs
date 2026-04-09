use anyhow::Result;
use tracing::info;

/// 执行初始化命令
pub async fn execute() -> Result<()> {
    match crate::config::AppConfig::create_example() {
        Ok(path) => {
            info!("✅ 配置文件已创建: {}", path.display());
            println!("\n请编辑配置文件，填入你的 AccessKey 和 Bucket 信息");
            println!("\n使用示例:");
            println!("  aliyun-oss-cli ls --profile hangzhou");
            println!("  aliyun-oss-cli upload myimage.jpg --profile hangzhou");
        }
        Err(e) => {
            eprintln!("❌ 创建配置文件失败: {}", e);
        }
    }
    Ok(())
}
