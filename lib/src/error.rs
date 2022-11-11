#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Chrono(#[from] chrono::ParseError),

    #[error("Expected IPLD String for timestamp")]
    ExpectedStringForTimestamp,

    #[error("Missing Field '{}'", .0)]
    MissingField(String),

    #[error("Field '{}' should be type '{}'", .0, .1)]
    WrongFieldType(String, String),

    #[error("Unexpected Type '{}'", .0)]
    UnexpectedType(String),
}
