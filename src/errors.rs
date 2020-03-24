pub type RescErr = Box<dyn std::error::Error>;
pub type RescResult<T> = Result<T, RescErr>;
