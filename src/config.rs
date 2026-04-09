use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 应用配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub endpoint: Option<String>,
    #[serde(rename = "access-key")]
    pub access_key: Option<String>,
    #[serde(rename = "secret-key")]
    pub secret_key: Option<String>,
    pub bucket: Option<String>,
    
    #[serde(skip)]
    pub config_file: Option<PathBuf>,
}

/// 配置文件结构（支持多 profile）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigFile {
    #[serde(flatten)]
    profiles: HashMap<String, AppConfig>,
}

impl AppConfig {
    /// 加载配置
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        // 1. 如果指定了配置文件，优先使用
        if let Some(path) = config_path {
            return Self::from_file(path);
        }

        // 2. 尝试从当前目录加载
        if Path::new("config.yaml").exists() {
            return Self::from_file(Path::new("config.yaml"));
        }

        // 3. 尝试从用户目录加载
        if let Some(home) = dirs::home_dir() {
            let config_dir = home.join(".aliyun-oss-cli");
            let config_file = config_dir.join("config.yaml");
            if config_file.exists() {
                return Self::from_file(&config_file);
            }
        }

        // 4. 返回空配置（从环境变量获取）
        Ok(Self::default())
    }

    /// 从文件加载配置（自动识别扁平格式和多 profile 格式）
    fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("读取配置文件失败: {}", path.display()))?;

        // 先尝试扁平格式（endpoint/access-key/... 直接在顶层）
        if let Ok(mut config) = serde_yaml::from_str::<AppConfig>(&content) {
            if config.endpoint.is_some() || config.access_key.is_some() {
                config.config_file = Some(path.to_path_buf());
                return Ok(config);
            }
        }

        // 再尝试多 profile 格式（default: { endpoint: ... }）
        let config_file: ConfigFile = serde_yaml::from_str(&content)
            .with_context(|| format!("解析配置文件失败: {}", path.display()))?;

        let mut config = config_file
            .profiles
            .get("default")
            .cloned()
            .unwrap_or_default();

        config.config_file = Some(path.to_path_buf());
        Ok(config)
    }

    /// 加载指定 profile 的配置
    pub fn load_with_profile(config_path: Option<&Path>, profile: &str) -> Result<Self> {
        let path = if let Some(p) = config_path {
            p.to_path_buf()
        } else if let Some(home) = dirs::home_dir() {
            home.join(".aliyun-oss-cli").join("config.yaml")
        } else {
            anyhow::bail!("无法找到配置文件");
        };

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("读取配置文件失败: {}", path.display()))?;

        // 扁平格式只有一个 profile，当请求 default 时直接返回
        if let Ok(flat) = serde_yaml::from_str::<AppConfig>(&content) {
            if flat.endpoint.is_some() || flat.access_key.is_some() {
                if profile == "default" {
                    let mut config = flat;
                    config.config_file = Some(path);
                    return Ok(config);
                }
                anyhow::bail!("配置文件为扁平格式，不支持 profile '{}'", profile);
            }
        }

        let config_file: ConfigFile = serde_yaml::from_str(&content)
            .with_context(|| format!("解析配置文件失败: {}", path.display()))?;

        let mut config = config_file
            .profiles
            .get(profile)
            .cloned()
            .with_context(|| format!("Profile '{}' 不存在", profile))?;

        config.config_file = Some(path);
        Ok(config)
    }

    /// 创建示例配置文件
    pub fn create_example() -> Result<PathBuf> {
        let home = dirs::home_dir().context("无法获取用户主目录")?;
        let config_dir = home.join(".aliyun-oss-cli");
        let config_path = config_dir.join("config.yaml");

        // 创建目录
        std::fs::create_dir_all(&config_dir)
            .context("创建配置目录失败")?;

        // 检查文件是否已存在
        if config_path.exists() {
            anyhow::bail!("配置文件已存在: {}", config_path.display());
        }

        let example_config = r#"# 阿里云 OSS CLI 配置文件
# 
# 配置说明：
# 1. 此文件支持多配置节，使用 --profile 指定
# 2. 环境变量优先级：命令行 > 环境变量 > 配置文件
# 3. 支持的环境变量：ALIYUN_OSS_ENDPOINT, ALIYUN_OSS_ACCESS_KEY, ALIYUN_OSS_SECRET_KEY, ALIYUN_OSS_BUCKET

# 默认配置
default:
  endpoint: "oss-cn-hangzhou.aliyuncs.com"
  access-key: ""
  secret-key: ""
  bucket: ""

# 杭州区域
hangzhou:
  endpoint: "oss-cn-hangzhou.aliyuncs.com"
  access-key: ""
  secret-key: ""
  bucket: "your-bucket-name"

# 北京区域
beijing:
  endpoint: "oss-cn-beijing.aliyuncs.com"
  access-key: ""
  secret-key: ""
  bucket: "your-bucket-name"

# 海外区域
overseas:
  endpoint: "oss-us-west-1.aliyuncs.com"
  access-key: ""
  secret-key: ""
  bucket: "your-bucket-name"
"#;

        std::fs::write(&config_path, example_config)
            .context("写入配置文件失败")?;

        Ok(config_path)
    }

    /// 验证配置是否完整
    pub fn validate(&self) -> Result<()> {
        if self.endpoint.is_none() {
            anyhow::bail!("未配置 endpoint，请通过 --endpoint、环境变量 ALIYUN_OSS_ENDPOINT 或配置文件指定");
        }
        if self.access_key.is_none() {
            anyhow::bail!("未配置 access-key，请通过 --access-key、环境变量 ALIYUN_OSS_ACCESS_KEY 或配置文件指定");
        }
        if self.secret_key.is_none() {
            anyhow::bail!("未配置 secret-key，请通过 --secret-key、环境变量 ALIYUN_OSS_SECRET_KEY 或配置文件指定");
        }
        Ok(())
    }
}
