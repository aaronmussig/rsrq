use redis::RedisError;

#[derive(Debug)]
pub enum RsrqError {
    RedisConnError(RedisError),
    RedisOpError(RedisError),
    CmdParserError(String),
    EnvError(std::env::VarError),
    JobNotFound(usize),
    SystemTimeError(std::time::SystemTimeError),
    ParserError(String),
    SnakemakeInvalidJobScript(String),
    InvalidJson(String),
    FileNotFound(std::io::Error),
    FileReadError(std::io::Error),
    IOError(std::io::Error),
    GeneralError(String),
}

impl std::fmt::Display for RsrqError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RsrqError::RedisConnError(e) => write!(f, "Redis connection error: {}", e),
            RsrqError::RedisOpError(e) => write!(f, "Redis operation error: {}", e),
            RsrqError::CmdParserError(e) => write!(f, "Command parser error: {}", e),
            RsrqError::EnvError(e) => write!(f, "Environment variable error: {}", e),
            RsrqError::JobNotFound(e) => write!(f, "Job not found: {}", e),
            RsrqError::SystemTimeError(e) => write!(f, "System time error: {}", e),
            RsrqError::ParserError(e) => write!(f, "Parser error: {}", e),
            RsrqError::SnakemakeInvalidJobScript(e) => write!(f, "Snakemake invalid job script: {}", e),
            RsrqError::InvalidJson(e) => write!(f, "Invalid JSON: {}", e),
            RsrqError::FileNotFound(e) => write!(f, "File not found: {}", e),
            RsrqError::FileReadError(e) => write!(f, "File read error: {}", e),
            RsrqError::IOError(e) => write!(f, "IO error: {}", e),
            RsrqError::GeneralError(e) => write!(f, "General error: {}", e),
        }
    }
}

impl std::error::Error for RsrqError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RsrqError::RedisConnError(e) => Some(e),
            RsrqError::RedisOpError(e) => Some(e),
            RsrqError::CmdParserError(_) => None,
            RsrqError::EnvError(e) => Some(e),
            RsrqError::JobNotFound(_) => None,
            RsrqError::SystemTimeError(e) => Some(e),
            RsrqError::ParserError(_) => None,
            RsrqError::SnakemakeInvalidJobScript(_) => None,
            RsrqError::InvalidJson(_) => None,
            RsrqError::FileNotFound(_) => None,
            RsrqError::FileReadError(_) => None,
            RsrqError::IOError(e) => Some(e),
            RsrqError::GeneralError(_) => None,
        }
    }
}
