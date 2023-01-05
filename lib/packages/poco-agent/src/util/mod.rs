pub fn pretty_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{} B", bytes);
    }
    let exp = (bytes as f64).log(1024.0).floor() as u32;
    let pre = "KMGTPE".as_bytes()[(exp - 1) as usize] as char;
    format!("{:.2} {}B", bytes as f64 / 1024f64.powi(exp as i32), pre)
}