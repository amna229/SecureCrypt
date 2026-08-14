pub fn generate_file(size: u64, unit: &str) -> Vec<u8> {
    
    let bytes = match unit {
        "KB" => size * 1024,
        "MB" => size * 1024 * 1024,
        "GB" => size * 1024 * 1024 * 1024,
        _ => panic!("Unknown file size unit"),
    };

    vec![0u8; bytes as usize]
}