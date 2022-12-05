#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Backend(#[from] crate::backend::implementation::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    GenericIrohError(#[from] anyhow::Error),

    #[error(transparent)]
    Time(#[from] time::error::Parse),

    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Expected IPLD String for timestamp")]
    ExpectedStringForTimestamp,

    #[error("Missing Field '{}'", .0)]
    MissingField(String),

    #[error("Field '{}' should be type '{}'", .0, .1)]
    WrongFieldType(String, String),

    #[error("Unexpected Type '{}'", .0)]
    UnexpectedType(String),
}
