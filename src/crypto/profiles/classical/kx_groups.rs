use rustls::crypto::SupportedKxGroup;
use rustls::crypto::aws_lc_rs;

/// Returns the key exchange groups supported by the classical profile.
///
/// The groups defined here are used during TLS negotiation when
/// the classical cryptographic profile is selected.
pub fn supported_kx_groups() -> Vec<&'static dyn SupportedKxGroup> {
    vec![
        aws_lc_rs::kx_group::SECP256R1,
        aws_lc_rs::kx_group::SECP384R1,
        aws_lc_rs::kx_group::X25519,
    ]
}





//Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classical_kx_groups_are_available() {
        let groups = supported_kx_groups();

        assert_eq!(format!("{:?}", groups[0].name()), "secp256r1");
        assert_eq!(format!("{:?}", groups[1].name()), "secp384r1");
        assert_eq!(format!("{:?}", groups[2].name()), "X25519");
    }
}