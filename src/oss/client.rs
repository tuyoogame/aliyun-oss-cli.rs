use bytes::Bytes;
use reqwest::{Client, Method, Response};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use super::error::OSSError;
use super::signature::{get_date_string, Signature};

/// OSS 客户端
pub struct OSSClient {
    client: Client,
    endpoint: String,
    access_key_id: String,
    access_key_secret: String,
    bucket: Option<String>,
    signature: Signature,
}

/// OSS 对象信息
#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub key: String,
    pub size: i64,
    pub last_modified: String,
    pub etag: String,
}

/// 列出对象的响应（OSS 返回 XML）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ListBucketResult {
    #[serde(default)]
    contents: Vec<Content>,
    #[serde(default)]
    common_prefixes: Vec<CommonPrefix>,
    is_truncated: Option<bool>,
    #[serde(default)]
    next_marker: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Content {
    key: String,
    size: i64,
    last_modified: String,
    #[serde(rename = "ETag")]
    etag: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CommonPrefix {
    prefix: String,
}

impl OSSClient {
    /// 创建新的 OSS 客户端
    pub fn new(
        endpoint: String,
        access_key_id: String,
        access_key_secret: String,
    ) -> Result<Self, OSSError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?;

        let signature = Signature::new(
            access_key_id.clone(),
            access_key_secret.clone(),
        );

        Ok(Self {
            client,
            endpoint,
            access_key_id,
            access_key_secret,
            bucket: None,
            signature,
        })
    }

    /// 设置 bucket
    pub fn with_bucket(mut self, bucket: String) -> Self {
        self.bucket = Some(bucket);
        self
    }

    /// 获取 bucket 名称
    pub fn get_bucket(&self) -> Result<&String, OSSError> {
        self.bucket.as_ref()
            .ok_or_else(|| OSSError::ConfigError("未设置 bucket".to_string()))
    }

    /// 发送请求
    async fn request(
        &self,
        method: Method,
        object_key: Option<&str>,
        headers: Option<HashMap<String, String>>,
        query: Option<HashMap<String, String>>,
        body: Option<Bytes>,
    ) -> Result<Response, OSSError> {
        let bucket = self.get_bucket()?;
        
        // 构建 URL
        let url = if let Some(key) = object_key {
            format!("https://{}.{}/{}", bucket, self.endpoint, key)
        } else {
            format!("https://{}.{}", bucket, self.endpoint)
        };

        // 构建请求
        let mut request_builder = self.client.request(method.clone(), &url);

        // 添加查询参数
        if let Some(q) = query {
            request_builder = request_builder.query(&q);
        }

        // 添加请求头
        let date = get_date_string();
        let mut all_headers = headers.unwrap_or_default();
        all_headers.insert("Date".to_string(), date.clone());

        // 计算签名
        let content_type = all_headers.get("Content-Type").map(|s| s.as_str());
        let resource = format!("/{}/{}", bucket, object_key.unwrap_or(""));
        let authorization = self.signature.generate(
            method.as_str(),
            &resource,
            content_type,
            &date,
        )?;
        all_headers.insert("Authorization".to_string(), authorization);

        for (key, value) in all_headers {
            request_builder = request_builder.header(key, value);
        }

        // 添加请求体
        if let Some(b) = body {
            request_builder = request_builder.body(b);
        }

        // 发送请求
        let response = request_builder.send().await?;

        // 检查响应状态
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(OSSError::OSSServiceError {
                code: status.to_string(),
                message: text,
            });
        }

        Ok(response)
    }

    /// 上传文件
    pub async fn upload_file(
        &self,
        object_key: &str,
        file_path: &Path,
        content_type: Option<&str>,
    ) -> Result<(), OSSError> {
        let data = tokio::fs::read(file_path).await?;
        let content_type = content_type
            .map(|s| s.to_string())
            .unwrap_or_else(|| mime_guess::from_path(file_path).first_or_octet_stream().to_string());

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type);
        headers.insert("Content-Length".to_string(), data.len().to_string());

        self.request(
            Method::PUT,
            Some(object_key),
            Some(headers),
            None,
            Some(Bytes::from(data)),
        ).await?;

        Ok(())
    }

    /// 上传数据
    pub async fn upload_data(
        &self,
        object_key: &str,
        data: Bytes,
        content_type: &str,
    ) -> Result<(), OSSError> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        headers.insert("Content-Length".to_string(), data.len().to_string());

        self.request(
            Method::PUT,
            Some(object_key),
            Some(headers),
            None,
            Some(data),
        ).await?;

        Ok(())
    }

    /// 列出对象
    pub async fn list_objects(
        &self,
        prefix: Option<&str>,
        max_keys: i32,
        marker: Option<&str>,
    ) -> Result<(Vec<ObjectInfo>, Vec<String>, Option<String>), OSSError> {
        let mut query = HashMap::new();
        query.insert("max-keys".to_string(), max_keys.to_string());
        
        if let Some(p) = prefix {
            query.insert("prefix".to_string(), p.to_string());
        }
        if let Some(m) = marker {
            query.insert("marker".to_string(), m.to_string());
        }

        let response = self.request(
            Method::GET,
            None,
            None,
            Some(query),
            None,
        ).await?;

        let xml_text = response.text().await
            .map_err(|e| OSSError::Other(format!("读取响应体失败: {}", e)))?;
        let result: ListBucketResult = quick_xml::de::from_str(&xml_text)
            .map_err(|e| OSSError::Other(format!("解析 XML 响应失败: {}", e)))?;

        let objects: Vec<ObjectInfo> = result.contents.into_iter()
            .map(|c| ObjectInfo {
                key: c.key,
                size: c.size,
                last_modified: c.last_modified,
                etag: c.etag,
            })
            .collect();

        let prefixes: Vec<String> = result.common_prefixes.into_iter()
            .map(|p| p.prefix)
            .collect();

        Ok((objects, prefixes, result.next_marker))
    }

    /// 检查对象是否存在
    pub async fn object_exists(&self, object_key: &str) -> Result<bool, OSSError> {
        let response = self.request(
            Method::HEAD,
            Some(object_key),
            None,
            None,
            None,
        ).await;

        match response {
            Ok(_) => Ok(true),
            Err(OSSError::OSSServiceError { code, .. }) if code == "404" => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// 删除对象
    pub async fn delete_object(&self, object_key: &str) -> Result<(), OSSError> {
        self.request(
            Method::DELETE,
            Some(object_key),
            None,
            None,
            None,
        ).await?;

        Ok(())
    }

    /// 获取对象元信息
    pub async fn get_object_meta(&self, object_key: &str) -> Result<HashMap<String, String>, OSSError> {
        let response = self.request(
            Method::HEAD,
            Some(object_key),
            None,
            None,
            None,
        ).await?;

        let mut meta = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                meta.insert(key.to_string(), v.to_string());
            }
        }

        Ok(meta)
    }

    /// 生成签名 URL
    pub fn generate_signed_url(
        &self,
        object_key: &str,
        expires: u64,
    ) -> Result<String, OSSError> {
        let bucket = self.get_bucket()?;
        self.signature.generate_signed_url(
            &self.endpoint,
            bucket,
            object_key,
            expires,
        )
    }

    /// 生成上传签名 URL
    pub fn generate_upload_signed_url(
        &self,
        object_key: &str,
        expires: u64,
        content_type: &str,
    ) -> Result<String, OSSError> {
        let bucket = self.get_bucket()?;
        self.signature.generate_upload_signed_url(
            &self.endpoint,
            bucket,
            object_key,
            expires,
            content_type,
        )
    }
}
