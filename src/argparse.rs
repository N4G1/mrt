use super::config;
use super::config::configmodels::*;
use crate::mrt_errors::MrtError;
use clap::{ArgMatches, Values};
use std::env;
use std::io::Result;

#[derive(Debug, PartialOrd, PartialEq)]
pub struct ParsedArgs {
    pub tags: Vec<String>,
    pub before_tags: Vec<String>,
    pub after_tags: Vec<String>,
}

pub const TAG_PREFIX: &str = "+";
pub const ADD_TAG_ARG: &str = "add-tag";
pub const DEL_TAG_ARG: &str = "del-tag";
pub const PARALLEL_TAG: &str = "parallel";
pub const LIST_TAGS_ARG: &str = "list-tags";
pub const CONTINUOUS_OUTPUT_ARG: &str = "continuous-output";

/// Takes in full list of arguments and returns tuple where
/// first element is tags found at start of arguments and
/// second element is the remaining arguments.
fn find_tags_in_args(args: &Vec<String>) -> ParsedArgs {
    let empty = ParsedArgs {
        tags: Vec::new(),
        before_tags: Vec::new(),
        after_tags: Vec::new(),
    };

    args.into_iter().fold(empty, |mut acc, arg| {
        match arg {
            a if arg.starts_with(TAG_PREFIX) && acc.after_tags.is_empty() => {
                acc.tags.push(a.clone())
            }
            a if !acc.tags.is_empty() => acc.after_tags.push(a.clone()),
            a => acc.before_tags.push(a.clone()),
        };
        acc
    })
}

/// Returns tuple with results.
/// First element is tags found at start of arguments.
/// Second element is the remaining arguments.
pub fn parse_arguments() -> ParsedArgs {
    let args = std::env::args();
    let args_vec = args.collect();
    find_tags_in_args(&args_vec)
}

pub fn handle_args_to_self(
    args: &ArgMatches,
    config: ConfigFile,
) -> std::result::Result<ConfigFile, MrtError> {
    let config_with_added = match args.values_of(ADD_TAG_ARG) {
        Some(tags) => add_tag_to_current_dir(tags, config),
        None => Ok(config),
    };

    let config_with_removed =
        config_with_added.and_then(|conf| match args.values_of(DEL_TAG_ARG) {
            Some(tags) => remove_tag_from_current_dir(tags, conf),
            None => Ok(conf),
        });

    match config_with_removed {
        Ok(conf) => {
            if args.is_present(LIST_TAGS_ARG) {
                println!("Config Version: {}", conf.version);
                for (tag_name, tag) in &conf.tags {
                    println!("{}:", tag_name);
                    for path in &tag.paths {
                        println!("\t{}", path);
                    }
                }
            }
            Ok(conf)
        }
        Err(err) => Err(MrtError::from(err)),
    }
}

fn add_tag_to_current_dir(tags: Values, mut config: ConfigFile) -> Result<ConfigFile> {
    for tag in tags {
        let current_path = env::current_dir()?;
        let cp = String::from(current_path.to_str().unwrap_or(""));

        let inserted_tag = config
            .tags
            .entry(tag.to_string())
            .or_insert(Tag { paths: vec![] });
        inserted_tag.paths.push(cp);
        inserted_tag.paths.sort();
        inserted_tag.paths.dedup();
    }
    config::loader::save_config(config)
}

fn remove_tag_from_current_dir(tags: Values, mut config: ConfigFile) -> Result<ConfigFile> {
    for tag in tags {
        let current_path = env::current_dir()?;
        let cp = String::from(current_path.to_str().unwrap_or(""));
        let tag_to_remove_path_from = config.tags.get_mut(tag);

        match tag_to_remove_path_from {
            Some(tag) => tag.paths.retain(|path| *path != cp),
            _ => println!("Didn't exist as tag /shrug"),
        }
    }
    config::loader::save_config(config)
}

#[cfg(test)]
mod test {

    use super::*;

    fn to_string_vec(v: Vec<&str>) -> Vec<String> {
        v.into_iter().map(|s| s.to_owned()).collect()
    }

    #[test]
    fn test_single_tag_is_parsed_correctly() {
        let test_args: Vec<String> = to_string_vec(vec!["mrt", "-p", "+testtag", "ls", "-l", "-h"]);

        let expected = ParsedArgs {
            tags: to_string_vec(vec!["+testtag"]),
            before_tags: to_string_vec(vec!["mrt", "-p"]),
            after_tags: to_string_vec(vec!["ls", "-l", "-h"]),
        };

        let result = find_tags_in_args(&test_args);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_tags_are_parsed_correctly() {
        let test_args: Vec<String> = to_string_vec(vec![
            "mrt", "+testtag", "+testaru", "+testari", "+x", "ls", "-l", "-h",
        ]);

        let expected = ParsedArgs {
            tags: to_string_vec(vec!["+testtag", "+testaru", "+testari", "+x"]),
            before_tags: to_string_vec(vec!["mrt"]),
            after_tags: to_string_vec(vec!["ls", "-l", "-h"]),
        };

        let result = find_tags_in_args(&test_args);

        assert_eq!(result, expected);
    }
}
