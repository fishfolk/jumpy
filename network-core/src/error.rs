pub type Result<T> = std::result::Result<T, Error>;

// TODO: Make it possible to distinguish between network errors and api status errors
pub type Error = &'static str;
