/// Generates a block of data with the requested size.
///
/// The content is filled with zeros because the purpose of the
/// transfer is to measure communication and cryptographic costs,
/// not the cost of generating real file contents.
pub fn generate_file(size: u64) -> Vec<u8> {
    vec![0u8; size as usize]
}
