use rustls::NamedGroup;


pub fn supported_kx_groups() -> &'static [NamedGroup] {

    &[
        NamedGroup::secp256r1,
        NamedGroup::secp384r1,
        NamedGroup::secp521r1,
        NamedGroup::X25519,
        NamedGroup::X448,
        NamedGroup::FFDHE2048,
        NamedGroup::FFDHE3072,
        NamedGroup::FFDHE4096,
        NamedGroup::FFDHE6144,
        NamedGroup::FFDHE8192,
    ]

}