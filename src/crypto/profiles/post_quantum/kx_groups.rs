use rustls::crypto::SupportedKxGroup;

use rustls::crypto::aws_lc_rs;

/// Returns the key exchange groups supported by the post-quantum profile.
///
/// The groups defined here are used during TLS negotiation when
/// the post-quantum cryptographic profile is selected.
pub fn supported_kx_groups() -> Vec<&'static dyn SupportedKxGroup> {
    vec![
        aws_lc_rs::kx_group::MLKEM768,
        aws_lc_rs::kx_group::MLKEM1024,
    ]
}





//Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_quantum_kx_groups_are_available() {
        let groups = supported_kx_groups();

        assert_eq!(groups.len(), 2);
        assert_eq!(format!("{:?}", groups[0].name()), "MLKEM768");
        assert_eq!(format!("{:?}", groups[1].name()), "MLKEM1024");
    }
}