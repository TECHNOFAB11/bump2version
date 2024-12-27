use crate::cli::Cli;
use crate::config::{Config, PartType};
use regex::Regex;
use std::cmp::min;
use std::collections::HashMap;
use std::ops::Index;
use tracing::{error, trace};

pub fn attempt_version_bump(args: Cli, config: Config) -> Option<String> {
    let parse_regex = config.parse;
    let regex = match Regex::new(&parse_regex) {
        Ok(r) => r,
        Err(err) => {
            error!(?err, parse_regex, "Invalid 'parse' regex");
            return None;
        }
    };

    let current_version = config.current_version;
    let mut parsed: HashMap<String, String> = HashMap::new();

    if let Some(captures) = regex.captures(&current_version) {
        for name in regex.capture_names().flatten() {
            if let Some(capture) = captures.name(name) {
                parsed.insert(name.to_string(), capture.as_str().to_string());
            }
        }
    }

    let order: Vec<&str> = config
        .serialize
        .match_indices('{')
        .map(|(i, _)| config.serialize[i + 1..].split('}').next().unwrap().trim())
        .collect();

    trace!(?order, "detected version parts");

    let mut bumped = false;

    let part_configs = config.part.clone();

    for label in order.clone() {
        if let Some(part) = parsed.get_mut(label) {
            let part_cfg = part_configs.as_ref().and_then(|c| c.get(label));

            if label == args.bump {
                match part_cfg
                    .map(|cfg| cfg.r#type.clone())
                    .unwrap_or(PartType::Number)
                {
                    PartType::String => {
                        let values = part_cfg
                            .unwrap()
                            .values
                            .clone()
                            .expect("part values do not exist for string type");
                        let old_index: usize = values
                            .iter()
                            .position(|val| val == part)
                            .expect("part value does not exist");
                        let new_index: usize = min(old_index + 1, values.len() - 1);
                        *part = values.index(new_index).to_string();
                        bumped = true;
                    }
                    PartType::Number => {
                        if let Ok(old_value) = part.parse::<u64>() {
                            *part = (old_value + 1).to_string();
                            bumped = true;
                        } else {
                            error!(part, "Failed to parse as u64");
                            return None;
                        }
                    }
                }
            } else if bumped {
                match part_cfg
                    .map(|cfg| cfg.r#type.clone())
                    .unwrap_or(PartType::Number)
                {
                    PartType::Number => *part = "0".to_string(),
                    PartType::String => {
                        let values = part_cfg
                            .unwrap()
                            .values
                            .clone()
                            .expect("part values do not exist for string type");
                        *part = values.index(0).to_string();
                    }
                }
            }
        } else {
            trace!(label, "part not found");
        }
    }

    if bumped {
        let mut new_version = config.serialize.clone();
        for part in order {
            trace!(new_version, part, "building new version");
            new_version = new_version.replace(
                &format!("{{{}}}", part),
                parsed.get(part).expect("unexpected part in version found"),
            );
        }
        trace!(new_version, "created new version");
        Some(new_version)
    } else {
        None
    }
}
