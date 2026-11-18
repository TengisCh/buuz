// SPDX-License-Identifier: Apache-2.0
// Copyright 2009 Odbayar Nyamtseren <odbayar.n@gmail.com> (original Buuz rules)
// Rust port: Copyright 2026 the Buuz contributors
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

//! Offline, platform-independent Latin -> Mongolian Cyrillic phonetic
//! transliteration core, behaviour-compatible with the original Buuz IME.
//!
//! ```
//! assert_eq!(buuz_core::convert("buuz id'ye"), "бууз идье");
//! assert_eq!(buuz_core::convert("sain baina uu"), "сайн байна уу");
//! ```

mod abbrev;
mod composer;
mod config;
mod keys;
mod rules;

pub use abbrev::{AbbrevError, Abbreviations};
pub use composer::{
    convert, convert_with, is_input_char, Composer, Key, KeyOutcome, Options, MAX_COMP_LEN,
};
pub use config::{
    Config, ConfigError, ConfigWarning, Parsed, Preset, CONFIG_VERSION, MAX_CONFIG_BYTES,
    MAX_SECTION_ENTRIES, MAX_VALUE_CHARS,
};
pub use keys::{KeyError, KeyMap};
pub use rules::export_rules;
