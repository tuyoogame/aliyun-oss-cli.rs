use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha1::Sha1;

use super::error::OSSError;

/// OSS 签名计算
pub struct Signature {
    access_key_id: String,
    access_key_secret: String,
}

type HmacSha1 = Hmac<Sha1>;

impl Signature {
    pub fn new(access_key_id: String, access_key_secret: String) -> Self {
        Self {
            access_key_id,
            access_key_secret,
        }
    }

    /// 生成签名
    /// 
    /// 签名格式: "OSS " + AccessKeyId + ":" + Signature
    pub fn generate(
        &self,
        method: &str,
        resource: &str,
        content_type: Option<&str>,
        date: &str,
    ) -> Result<String, OSSError> {
        // 构建签名字符串
        let mut string_to_sign = format!("{}\n\n", method);
        
        // Content-Type
        if let Some(ct) = content_type {
            string_to_sign.push_str(ct);
        }
        string_to_sign.push('\n');
        
        // Date
        string_to_sign.push_str(date);
        string_to_sign.push('\n');
        
        // CanonicalizedOSSHeaders (这里简化处理，实际需要处理 x-oss-* 头)
        
        // CanonicalizedResource
        string_to_sign.push_str(resource);

        // 计算 HMAC-SHA1
        let mut mac = HmacSha1::new_from_slice(self.access_key_secret.as_bytes())
            .map_err(|e| OSSError::SignatureError(e.to_string()))?;
        mac.update(string_to_sign.as_bytes());
        let result = mac.finalize();
        let signature = BASE64.encode(result.into_bytes());

        Ok(format!("OSS {}:{}", self.access_key_id, signature))
    }

    /// 生成签名 URL
    pub fn generate_signed_url(
        &self,
        endpoint: &str,
        bucket: &str,
        object_key: &str,
        expires: u64,
    ) -> Result<String, OSSError> {
        let date = Utc::now().timestamp() as u64;
        let expiration = date + expires;

        // 构建签名字符串
        let resource = format!("/{}/{}", bucket, object_key);
        let string_to_sign = format!("GET\n\n\n{}\n{}", expiration, resource);

        // 计算 HMAC-SHA1
        let mut mac = HmacSha1::new_from_slice(self.access_key_secret.as_bytes())
            .map_err(|e| OSSError::SignatureError(e.to_string()))?;
        mac.update(string_to_sign.as_bytes());
        let result = mac.finalize();
        let signature = BASE64.encode(result.into_bytes());

        // 构建 URL
        let encoded_key = urlencoding::encode(object_key);
        let url = format!(
            "https://{}.{}/{}?OSSAccessKeyId={}&Expires={}&Signature={}",
            bucket,
            endpoint,
            encoded_key,
            urlencoding::encode(&self.access_key_id),
            expiration,
            urlencoding::encode(&signature)
        );

        Ok(url)
    }

    /// 生成上传签名 URL
    pub fn generate_upload_signed_url(
        &self,
        endpoint: &str,
        bucket: &str,
        object_key: &str,
        expires: u64,
        content_type: &str,
    ) -> Result<String, OSSError> {
        let date = Utc::now().timestamp() as u64;
        let expiration = date + expires;

        // 构建签名字符串
        let resource = format!("/{}/{}", bucket, object_key);
        let string_to_sign = format!("PUT\n\n{}\n{}\n{}", content_type, expiration, resource);

        // 计算 HMAC-SHA1
        let mut mac = HmacSha1::new_from_slice(self.access_key_secret.as_bytes())
            .map_err(|e| OSSError::SignatureError(e.to_string()))?;
        mac.update(string_to_sign.as_bytes());
        let result = mac.finalize();
        let signature = BASE64.encode(result.into_bytes());

        // 构建 URL
        let encoded_key = urlencoding::encode(object_key);
        let url = format!(
            "https://{}.{}/{}?OSSAccessKeyId={}&Expires={}&Signature={}",
            bucket,
            endpoint,
            encoded_key,
            urlencoding::encode(&self.access_key_id),
            expiration,
            urlencoding::encode(&signature)
        );

        Ok(url)
    }
}

/// 获取 RFC1123 格式的日期字符串
pub fn get_date_string() -> String {
    Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}
