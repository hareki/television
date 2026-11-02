use std::hash::Hash;

use crate::{event::Key, utils::hashmaps};
use crossterm::event::{KeyCode, KeyModifiers};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[serde(default)]
pub struct ShellIntegrationConfig {
    /// {command: channel}
    #[serde(skip)]
    pub commands: FxHashMap<String, String>,
    pub fallback_channel: String,
    pub keybindings: FxHashMap<String, Key>,
}

impl<'de> Deserialize<'de> for ShellIntegrationConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct DeSerHelper {
            channel_triggers: FxHashMap<String, Vec<String>>,
            fallback_channel: String,
            keybindings: FxHashMap<String, Key>,
        }

        let helper = DeSerHelper::deserialize(deserializer)?;

        Ok(Self {
            commands: hashmaps::invert_nested_hashmap(
                &helper.channel_triggers,
            ),
            fallback_channel: helper.fallback_channel,
            keybindings: helper.keybindings,
        })
    }
}

impl Hash for ShellIntegrationConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // we're not actually using this for hashing, so this really only is a placeholder
        state.write_u8(0);
    }
}

const SMART_AUTOCOMPLETE_CONFIGURATION_KEY: &str = "smart_autocomplete";
const COMMAND_HISTORY_CONFIGURATION_KEY: &str = "command_history";
const DEFAULT_SHELL_AUTOCOMPLETE_KEY: char = 'T';
const DEFAULT_COMMAND_HISTORY_KEY: char = 'R';

impl ShellIntegrationConfig {
    // based on the keybindings configuration provided in the configuration file
    // (if any), extract the character triggers shell autocomplete
    pub fn get_shell_autocomplete_keybinding_character(&self) -> char {
        match self.keybindings.get(SMART_AUTOCOMPLETE_CONFIGURATION_KEY) {
            Some(&key) => extract_ctrl_char(key)
                .unwrap_or(DEFAULT_SHELL_AUTOCOMPLETE_KEY),
            None => DEFAULT_SHELL_AUTOCOMPLETE_KEY,
        }
    }
    // based on the keybindings configuration provided in the configuration file
    // (if any), extract the character triggers command history management
    // through tv
    pub fn get_command_history_keybinding_character(&self) -> char {
        match self.keybindings.get(COMMAND_HISTORY_CONFIGURATION_KEY) {
            Some(&key) => {
                extract_ctrl_char(key).unwrap_or(DEFAULT_COMMAND_HISTORY_KEY)
            }
            None => DEFAULT_COMMAND_HISTORY_KEY,
        }
    }
}

/// Extract an upper-case character from a `Key` if it is a single CTRL key
/// (or CTRL-Space).  Returns `None` otherwise.
fn extract_ctrl_char(key: Key) -> Option<char> {
    if key.modifiers == KeyModifiers::CONTROL {
        match key.code {
            KeyCode::Char(' ') => Some(' '),
            KeyCode::Char(c) => Some(c.to_ascii_uppercase()),
            _ => None,
        }
    } else {
        None
    }
}
