use std::error::Error;

#[derive(Debug)]
pub struct AppError {
    kind: Kind,
    code: u16,
}

impl AppError {
    pub fn new(kind: Kind) -> Self {
        let code = match kind {
            Kind::InvalidFlags => 101,
        };
        Self { kind, code }
    }
}

impl Error for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let des: String = match self.kind {
            Kind::InvalidFlags => "Invalid Flags".to_owned()
        };
        write!(f, "{}:{}", self.code, &des)
    }
}

#[derive(Debug)]
pub enum Kind {
    InvalidFlags,
}