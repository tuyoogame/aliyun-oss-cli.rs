use thiserror::Error;

#[derive(Error, Debug)]
pub enum OSSError {
    #[error("HTTP 请求失败: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("签名失败: {0}")]
    SignatureError(String),
    
    #[error("OSS 错误: {code} - {message}")]
    OSSServiceError { code: String, message: String },
    
    #[error("文件不存在: {0}")]
    ObjectNotFound(String),
    
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("其他错误: {0}")]
    Other(String),
}

impl From<String> for OSSError {
    fn from(s: String) -> Self {
        OSSError::Other(s)
    }
}
