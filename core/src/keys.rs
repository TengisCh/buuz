// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Custom key mappings: the user's own keys on top of the built-in table.

use std::fmt;

use crate::composer::is_input_char;
use crate::rules::{build_rules, find_target, to_lower, Donor, RuleSet};

/// A fault in a key mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyError {
    /// The key is not 1 to 3 input characters.
    BadKey { key: String },
    /// No built-in key gives this target.
    UnknownTarget { key: String, value: String },
    /// The target has upper case letters. Targets are lower case.
    UpperCaseTarget { key: String, value: String },
}

impl fmt::Display for KeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyError::BadKey { key } => {
                write!(f, "invalid key {key:?} (use 1 to 3 input keys)")
            }
            KeyError::UnknownTarget { key, value } => write!(
                f,
                "key {key:?}: no built-in key gives {value:?} (use a letter the built-in keys give, or a pair such as \"у/ү\")"
            ),
            KeyError::UpperCaseTarget { key, value } => write!(
                f,
                "key {key:?}: the target {value:?} must be lower case (upper case follows the key you press)"
            ),
        }
    }
}

impl std::error::Error for KeyError {}

/// Check a key and a target. An empty target removes the key.
pub(crate) fn check_mapping(key: &str, value: &str) -> Result<(), KeyError> {
    let n = key.chars().count();
    if !(1..=3).contains(&n) || !key.chars().all(is_input_char) {
        return Err(KeyError::BadKey {
            key: key.to_string(),
        });
    }
    if value.chars().any(|c| to_lower(c) != c) {
        return Err(KeyError::UpperCaseTarget {
            key: key.to_string(),
            value: value.to_string(),
        });
    }
    if !value.is_empty() && find_target(value).is_none() {
        return Err(KeyError::UnknownTarget {
            key: key.to_string(),
            value: value.to_string(),
        });
    }
    Ok(())
}

/// A full rule table: the built-in rules with the user's changes applied.
/// Build it once (it is not cheap) and share it in [`crate::Options::keys`];
/// the composer then looks keys up in it exactly as in the built-in table.
pub struct KeyMap {
    rules: RuleSet,
}

impl fmt::Debug for KeyMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyMap").finish_non_exhaustive()
    }
}

impl KeyMap {
    /// Build a key map from `(key, target)` pairs. The key is 1 to 3 input
    /// characters; case does not matter. The target names a Cyrillic letter or
    /// text that the built-in keys already give (`ү`, `х`), or a gender pair
    /// (`у/ү`). The new key copies every rule of the built-in key that gives
    /// the target, so vowel harmony still works. An empty target removes the
    /// key. A later pair for the same key wins.
    pub fn from_pairs(pairs: &[(String, String)]) -> Result<KeyMap, KeyError> {
        let mut changes = Vec::with_capacity(pairs.len());
        for (key, value) in pairs {
            check_mapping(key, value)?;
            let lower: Vec<char> = key.chars().map(to_lower).collect();
            let donor = if value.is_empty() {
                Donor::default()
            } else {
                find_target(value).unwrap_or_default()
            };
            changes.push((lower, donor));
        }
        Ok(KeyMap {
            rules: build_rules(&changes),
        })
    }

    pub(crate) fn rules(&self) -> &RuleSet {
        &self.rules
    }
}
