use std::path::Path;

/// 格式化文件大小
pub fn format_size(size: i64) -> String {
    const UNIT: i64 = 1024;
    
    if size < UNIT {
        return format!("{} B", size);
    }
    
    let units = ["KB", "MB", "GB", "TB", "PB"];
    let mut size = size as f64;
    let mut unit_index = 0;
    
    while size >= UNIT as f64 && unit_index < units.len() - 1 {
        size /= UNIT as f64;
        unit_index += 1;
    }
    
    format!("{:.1} {}", size, units[unit_index])
}

/// 判断是否是图片文件
pub fn is_image(filename: &str) -> bool {
    let image_exts = ["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg", "ico", "tif", "tiff"];
    
    if let Some(ext) = Path::new(filename).extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        image_exts.contains(&ext.as_str())
    } else {
        false
    }
}

/// 判断 Content-Type 是否是图片
pub fn is_image_content_type(content_type: &str) -> bool {
    content_type.starts_with("image/")
}

/// 打开浏览器
#[cfg(target_os = "macos")]
pub fn open_browser(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::process::Command::new("open")
        .arg(url)
        .spawn()?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn open_browser(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::process::Command::new("cmd")
        .args(["/c", "start", url])
        .spawn()?;
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn open_browser(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()?;
    Ok(())
}

/// 获取 Content-Type
pub fn get_content_type(filename: &str) -> String {
    mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string()
}
