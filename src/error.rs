use snafu::{Location, Snafu};

#[derive(Snafu, Debug)]
pub enum BotError {
    #[snafu(transparent)]
    ReqwestError {
        #[snafu(implicit)]
        loc: Location,
        source: reqwest::Error,
    },
    #[snafu(transparent)]
    JemallocCtlError {
        #[snafu(implicit)]
        loc: Location,
        source: tikv_jemalloc_ctl::Error,
    },
    #[snafu(transparent)]
    SeaOrmError {
        #[snafu(implicit)]
        loc: Location,
        source: sea_orm::DbErr,
    },
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
        #[snafu(source(from(serenity::Error, Box::new)))]
        source: Box<serenity::Error>,
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
