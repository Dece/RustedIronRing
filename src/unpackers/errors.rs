use std::io;

#[derive(Debug)]
pub enum UnpackError {
    Io(io::Error),
    Parsing(String),
    Compression(String),
    Naming(String),
    Unknown(String),
}

impl UnpackError {
    pub fn parsing_err(filetype: &str, kind: nom::error::ErrorKind) -> UnpackError {
        let message = format!("{} parsing failed: {}", filetype, kind.description());
        UnpackError::Parsing(message)
    }
}

impl From<io::Error> for UnpackError {
    fn from(e: io::Error) -> Self {
        UnpackError::Io(e)
    }
}
