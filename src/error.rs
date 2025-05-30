use snafu::{Location, Snafu};

#[derive(Snafu, Debug)]
pub enum BotError {
    #[snafu(transparent)]
    IoError {
        #[snafu(implicit)]
        loc: Location,
        source: std::io::Error,
    },
    #[snafu(transparent)]
    SerenityError {
        #[snafu(implicit)]
        loc: Location,
        source: serenity::Error,
    },
    #[snafu(whatever, display("{message}"))]
    GenericError {
        message: String,
        // Having a `source` is optional, but if it is present, it must
        // have this specific attribute and type:
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}
