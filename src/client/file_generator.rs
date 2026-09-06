/// Generates a block of data with the requested size.
///
/// The content is filled with zeros because the purpose of the
/// transfer is to measure communication and cryptographic costs,
/// not the cost of generating real file contents.
pub fn generate_file(size: u64) -> Vec<u8> {
    vec![0u8; size as usize]
}





//Unit tests
#[cfg(test)]
mod tests {
    use super::generate_file;

    #[test]
    fn generates_empty_file() {
        let data = generate_file(0);

        assert!(data.is_empty());
    }

    #[test]
    fn generates_file_with_requested_size() {
        let data = generate_file(1024);

        assert_eq!(data.len(), 1024);
    }

    #[test]
    fn generated_file_is_filled_with_zeros() {
        let data = generate_file(1024);

        assert!(data.iter().all(|byte| *byte == 0));
    }
}