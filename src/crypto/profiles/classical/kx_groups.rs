use rustls::crypto::aws_lc_rs;
use rustls::crypto::SupportedKxGroup;


pub fn supported_kx_groups() -> Vec<&'static dyn SupportedKxGroup> {

    vec![
        aws_lc_rs::kx_group::SECP256R1,
        aws_lc_rs::kx_group::SECP384R1,
        aws_lc_rs::kx_group::X25519,
    ]

}