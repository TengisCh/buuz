// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0.

//! Convert typed sentences, one per line on stdin, with the web defaults
//! (smart mode and the word list) and with autocorrect on. Use it to compare
//! real typing samples with what Buuz gives:
//!
//! ```sh
//! cargo run --release --example try_convert < samples.txt
//! ```

use std::io::BufRead;
use std::sync::Arc;

use buuz_core::{convert_with, Options, WordList};

fn main() {
    let words = Arc::new(
        WordList::parse(include_str!("../../data/wordlist/mn.txt")).expect("word list parses"),
    );
    let web = Options {
        words: Some(words.clone()),
        ..Options::smart()
    };
    let fix = Options {
        words: Some(words),
        autocorrect: true,
        ..Options::smart()
    };
    for line in std::io::stdin().lock().lines() {
        let line = line.expect("stdin is text");
        let a = convert_with(&format!("{line} "), &web);
        let b = convert_with(&format!("{line} "), &fix);
        println!(
            "{line}\n  web:         {}\n  autocorrect: {}",
            a.trim_end(),
            b.trim_end()
        );
    }
}
