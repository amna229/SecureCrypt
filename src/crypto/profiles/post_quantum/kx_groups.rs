use rustls::crypto::SupportedKxGroup;
use rustls::crypto::aws_lc_rs;

pub fn supported_kx_groups() -> Vec<&'static dyn SupportedKxGroup> {
    vec![
        aws_lc_rs::kx_group::MLKEM768,
        aws_lc_rs::kx_group::MLKEM1024,
    ]
}
