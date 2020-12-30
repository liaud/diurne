use anyhow::Context;
use serde::Deserialize;
use std::collections::HashMap;
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

fn main() -> anyhow::Result<()> {
    let matches = clap::App::new("diurne")
        .version("0.1")
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .long("config")
                .help("config file")
                .value_name("CONFIG")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let config_path = &Path::new(matches.value_of("config").unwrap());
    let parsed_config = parse_config(config_path).context("failed to parse the config file.")?;
    let config = validate_config(config_path, parsed_config).context("invalid config.")?;

    println!("config {:#?}", config);
    let database =
        ReportDatabase::with_config(&config).context("failed to open report database.")?;

    Ok(())
}

#[derive(Deserialize)]
pub struct ParsedConfig {
    tags: Vec<String>,
    aliases: HashMap<Box<str>, Vec<String>>,
}

pub type TagIndex = u8;

#[derive(Debug)]
pub struct Config {
    tags: Vec<Box<str>>,
    aliases: HashMap<Box<str>, Vec<TagIndex>>,
    database_path: Box<Path>,
}

fn parse_config(path: &Path) -> anyhow::Result<ParsedConfig> {
    let content = fs::read_to_string(path).context("failed to read config file.")?;
    toml::from_str(&content).context("failed to deserialize config file.")
}

#[derive(Debug, Error)]
pub enum ConfigValidationError {
    #[error("An unknown tag have been found: {name}.")]
    UnknownTag { name: String },
}

fn validate_config(
    config_path: &Path,
    mut parsed: ParsedConfig,
) -> Result<Config, ConfigValidationError> {
    parsed.tags.sort();
    parsed.tags.dedup();
    let tags = parsed.tags.into_iter().map(Box::from).collect::<Vec<_>>();
    let mut aliases = HashMap::with_capacity(parsed.aliases.len());
    for (alias, compound) in parsed.aliases {
        let tag_indices = compound
            .into_iter()
            .map(|tag| {
                let index = tags
                    .iter()
                    .position(|registered_tag| &registered_tag as &str == &tag)
                    .ok_or_else(|| ConfigValidationError::UnknownTag { name: tag })?;

                Ok(index as TagIndex)
            })
            .collect::<Result<Vec<TagIndex>, ConfigValidationError>>()?;

        aliases.insert(Box::from(alias), tag_indices);
    }

    let mut database_path = PathBuf::from(config_path);
    database_path.set_extension("db");

    Ok(Config {
        tags,
        aliases,
        database_path: Box::from(database_path),
    })
}

pub struct ReportDatabase {
    connection: rusqlite::Connection,
}

impl ReportDatabase {
    pub fn with_config(config: &Config) -> Result<Self, rusqlite::Error> {
        let connection = rusqlite::Connection::open(&config.database_path)?;
        connection.set_db_config(
            rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY,
            true,
        )?;

        Ok(ReportDatabase {
            connection: Self::insert_tables(connection)?,
        })
    }

    fn insert_tables(
        connection: rusqlite::Connection,
    ) -> Result<rusqlite::Connection, rusqlite::Error> {
        connection.execute(
            "CREATE TABLE IF NOT EXISTS tags(
                tagid INTEGER PRIMARY KEY,
                name TEXT UNIQUE
            );",
            rusqlite::NO_PARAMS,
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS transfers(
                transferid INTEGER PRIMARY KEY,
                store TEXT,
                amount INTEGER
            );",
            rusqlite::NO_PARAMS,
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS tagged_transfers(
                tagid INTEGER,
                transferid INTEGER,
                FOREIGN KEY(tagid) REFERENCES tags(tagid) ON DELETE CASCADE,
                FOREIGN KEY(transferid) REFERENCES transfers(transferid) ON DELETE CASCADE
            );
            CREATE UNIQUE INDEX IF NOT EXISTS tagged_transfers_lookup
                ON tagged_transfers(tagid, transferid);",
            rusqlite::NO_PARAMS,
        )?;

        Ok(connection)
    }
}
