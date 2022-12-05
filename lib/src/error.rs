#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Backend(#[from] crate::backend::implementation::Error),

    #[error(transparent)]
    GenericIrohError(#[from] anyhow::Error),

    #[error(transparent)]
    Time(#[from] time::error::Parse),

    #[error("Expected IPLD String for timestamp")]
    ExpectedStringForTimestamp,

    #[error("Missing Field '{}'", .0)]
    MissingField(String),

    #[error("Field '{}' should be type '{}'", .0, .1)]
    WrongFieldType(String, String),

    #[error("Unexpected Type '{}'", .0)]
    UnexpectedType(String),
}
