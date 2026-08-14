#[derive(Debug, serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CryptoMode {

    Classical,
    PostQuantum,
    
}



impl std::fmt::Display for CryptoMode {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        match self {

            Self::Classical => write!(f, "classical"),
            Self::PostQuantum => write!(f, "post_quantum"),
            
        }
    }
}