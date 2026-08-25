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
