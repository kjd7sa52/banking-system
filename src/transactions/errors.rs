#[derive(Debug)]
pub enum TransactionError {
    Denied(String),
    Rejected(String),
}

impl TransactionError {
    pub fn deny<T: Into<String>>(cause: T) -> Self {
        Self::Denied(cause.into())
    }

    pub fn reject<T: Into<String>>(cause: T) -> Self {
        Self::Rejected(cause.into())
    }
}
