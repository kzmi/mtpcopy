#[derive(Debug)]
pub struct AppError {
    pub message: String,
}

impl AppError {
    pub fn new<S: Into<String>>(message: S) -> AppError {
        AppError {
            message: message.into(),
        }
    }
}

impl std::error::Error for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
