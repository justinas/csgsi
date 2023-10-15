use std::sync::OnceLock;

use anyhow::{anyhow, bail};
use regex::Regex;

use crate::gsi::Team;

fn kill_regex() -> &'static Regex {
    static LOCK: OnceLock<Regex> = OnceLock::new();
    // `Name<id><wonid><team> [coordinates?]`
    // We do not capture coordinates.
    let player_id_regex = r#""([^<]+?)<([^>]+?)><([^>]+?)><([^>]+?)>" \[.+?\]"#;

    LOCK.get_or_init(|| {
        Regex::new(&format!(
            r#"{} killed {} with "([^"]+)""#,
            player_id_regex, player_id_regex
        ))
        .unwrap()
    })
}

#[derive(Debug, PartialEq)]
pub struct Player {
    pub name: String,
    pub team: Team,
}

/// https://developer.valvesoftware.com/wiki/HL_Log_Standard
#[derive(Debug, PartialEq)]
pub enum Event {
    // TODO: add assist support
    Kill {
        killer: Player,
        target: Player,
        weapon: String,
    },
}

impl TryFrom<&str> for Event {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // Expect `DATE - TIME - MESSAGE`
        let mut split = value.splitn(3, '-');
        let message = match (split.next(), split.next(), split.next()) {
            (_, _, Some(msg)) => msg,
            _ => bail!("Invalid format"),
        };

        if let Some(matches) = kill_regex().captures(message) {
            assert_eq!(
                matches.len(),
                kill_regex().captures_len(),
                "all groups should be non-optional"
            );
            return Ok(Event::Kill {
                killer: Player {
                    name: matches[1].into(),
                    team: Team::from_log_name(&matches[4])
                        .ok_or(anyhow!("invalid team: {}", matches[4].to_string()))?,
                },
                target: Player {
                    name: matches[5].into(),
                    team: Team::from_log_name(&matches[8])
                        .ok_or(anyhow!("invalid team: {}", matches[8].to_string()))?,
                },
                weapon: matches[9].into(),
            });
        }
        bail!("unrecognized log message: {}", value);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_killed() {
        println!("{}", super::kill_regex());
        let e = Event::try_from(r#"10/15/2023 - 16:15:10.799 - "Telsen<11><BOT><TERRORIST>" [32 -2371 -40] killed "Yanni<18><BOT><CT>" [11 -2129 -36] with "xm1014""#).unwrap();
        assert_eq!(
            e,
            Event::Kill {
                killer: Player {
                    name: "Telsen".into(),
                    team: Team::T,
                },
                target: Player {
                    name: "Yanni".into(),
                    team: Team::CT,
                },
                weapon: "xm1014".into(),
            }
        )
    }
}
