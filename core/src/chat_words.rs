// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 the Buuz contributors. Licensed under the Apache License,
// Version 2.0. See the LICENSE file in this crate.

//! Everyday chat words (issue #101). The word list comes from Wikipedia, so
//! words of daily talk are rare in it (өчигдөр has a count of 31) or missing
//! (уулзъя). [`crate::WordList::with_chat_words`] gives each of these words
//! at least [`CHAT_COUNT`], so word choice and autocorrect can pick them.
//! The list is hand-made from real chat typing. Keep it short and only add
//! a word when its usual misspellings do not hide another real word.

/// The smallest count that a chat word gets.
pub(crate) const CHAT_COUNT: u32 = 10_000;

pub(crate) const CHAT_WORDS: &[&str] = &[
    // Time
    "өчигдөр",
    "өнөөдөр",
    "маргааш",
    "нөгөөдөр",
    "уржигдар",
    "одоо",
    "дараа",
    "удахгүй",
    "өглөө",
    "өглөөний",
    "орой",
    "шөнө",
    "цагт",
    "есөн",
    "өвөл",
    // Questions
    "хаана",
    "хэзээ",
    "яагаад",
    "ямар",
    // Particles and short answers
    "шүү",
    "зүгээр",
    "ёстой",
    "үнэхээр",
    "гоё",
    "сайхан",
    "баярлалаа",
    "уучлаарай",
    "минь",
    // Verbs in everyday forms
    "уулзъя",
    "явъя",
    "ярья",
    "үзье",
    "идье",
    "ирье",
    "очъё",
    "болъё",
    "байя",
    "харьж",
    "харьсан",
    "ярьж",
    "ярьсан",
    "хүлээж",
    "хүлээгээрэй",
    "залгаарай",
    "бичээрэй",
    "авчраарай",
    "мэдэхгүй",
    "ойлголоо",
    "ирлээ",
    "явлаа",
    "бодоод",
    "гараад",
    "ороод",
    "ирээд",
    "очоод",
    // Things
    "гэртээ",
    "ажлаасаа",
    "бороо",
    "хүйтэн",
    "мөнгө",
    "дэлгүүр",
    "дэлгүүрээс",
    "хүүхдүүд",
];
