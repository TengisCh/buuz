// SPDX-License-Identifier: Apache-2.0
//! Latin to Cyrillic schemes for languages other than Mongolian.

use buuz_core::Scheme;

fn check(s: Scheme, cases: &[(&str, &str)]) {
    for (input, want) in cases {
        assert_eq!(s.convert(input), *want, "{} on {input:?}", s.id());
    }
}

#[test]
fn ids_round_trip() {
    for s in Scheme::ALL {
        assert_eq!(Scheme::from_id(s.id()), Ok(s));
    }
    assert!(Scheme::from_id("mn").is_err());
    assert_eq!(Scheme::for_language("uk"), Some(Scheme::Ukrainian));
    assert_eq!(Scheme::for_language("ru"), Some(Scheme::Russian));
    assert_eq!(Scheme::RussianGost.language(), "ru");
}

#[test]
fn drafts() {
    assert!(Scheme::Kazakh.is_draft());
    assert!(!Scheme::Russian.is_draft());
}

#[test]
fn russian_everyday() {
    check(
        Scheme::Russian,
        &[
            ("privet", "привет"),
            ("spasibo", "спасибо"),
            ("zdravstvuyte", "здравствуйте"),
            ("shchi", "щи"),
            ("yabloko", "яблоко"),
            ("yolka", "ёлка"),
            ("khorosho", "хорошо"),
            ("zhizn'", "жизнь"),
            ("pod''ezd", "подъезд"),
            ("krasivyy", "красивый"),
            ("moy", "мой"),
            ("segodnya", "сегодня"),
            ("chto eto", "что это"),
            ("e'kran", "экран"),
            ("taxi", "такси"),
            ("kak dela? 2 raza.", "как дела? 2 раза."),
        ],
    );
}

#[test]
fn russian_case() {
    check(
        Scheme::Russian,
        &[
            ("Moskva", "Москва"),
            ("MOSKVA", "МОСКВА"),
            ("Yuliya", "Юлия"),
            ("SHCHI", "ЩИ"),
            ("Zhizn'", "Жизнь"),
        ],
    );
}

#[test]
fn other_text_stays() {
    check(
        Scheme::Russian,
        &[("привет 123", "привет 123"), ("'a", "'а"), ("", "")],
    );
}

#[test]
fn russian_gost_system_b() {
    check(
        Scheme::RussianGost,
        &[
            ("shhuka", "щука"),
            ("czirk", "цирк"),
            ("ciklon", "циклон"),
            ("ob``ekt", "объект"),
            ("my`", "мы"),
            ("e`to", "это"),
            ("sel`", "сель"),
            ("Yuliya", "Юлия"),
        ],
    );
}

#[test]
fn ukrainian() {
    check(
        Scheme::Ukrainian,
        &[
            ("pryvit", "привіт"),
            ("dyakuyu", "дякую"),
            ("hlyboko", "глибоко"),
            ("gandzh", "ґандж"),
            ("yizhak", "їжак"),
            ("dialoh", "діалог"),
            ("pam''yat'", "пам\u{2bc}ять"),
            ("Kyyiv", "Київ"),
            ("ye", "є"),
            ("shchastya", "щастя"),
        ],
    );
}

#[test]
fn belarusian() {
    check(
        Scheme::Belarusian,
        &[
            ("dzyakuy", "дзякуй"),
            ("pryvitanne", "прывітанне"),
            ("wzmotsnic", "ўзмоцніц"),
            ("hety", "геты"),
        ],
    );
}

#[test]
fn bulgarian() {
    check(
        Scheme::Bulgarian,
        &[
            ("zdraveyte", "здравейте"),
            ("bu'lgarski", "български"),
            ("shtastie", "щастие"),
            ("qm", "ям"),
            ("yuli", "юли"),
            ("moy", "мой"),
        ],
    );
}

#[test]
fn serbian() {
    check(
        Scheme::Serbian,
        &[
            ("ljubav", "љубав"),
            ("njiva", "њива"),
            ("djak", "ђак"),
            ("đak", "ђак"),
            ("dobar dan", "добар дан"),
            ("šuma", "шума"),
            ("chaj", "чај"),
            ("cc", "ћ"),
            ("Dž", "Џ"),
            ("Njegos", "Његос"),
        ],
    );
}

#[test]
fn macedonian() {
    check(
        Scheme::Macedonian,
        &[
            ("gjubre", "ѓубре"),
            ("kje", "ќе"),
            ("dzvezda", "ѕвезда"),
            ("zdravo", "здраво"),
        ],
    );
}

#[test]
fn kazakh_draft() {
    check(
        Scheme::Kazakh,
        &[
            ("saelem", "сәлем"),
            ("rahmet", "рахмет"),
            ("jaqsy", "жақсы"),
            ("qazaq", "қазақ"),
        ],
    );
}

#[test]
fn kyrgyz_draft() {
    check(
        Scheme::Kyrgyz,
        &[
            ("kyrgyz", "кыргыз"),
            ("oeroen", "өрөн"),
            ("jakshy", "жакшы"),
            ("bashkarma", "башкарма"),
        ],
    );
}

#[test]
fn russian_volapuk() {
    check(
        Scheme::RussianVolapuk,
        &[
            ("npuBeT", "привет"),
            ("4to", "что"),
            ("kak dela", "как дела"),
            ("6e3", "без"),
            ("mo10", "мою"),
            ("b|", "ы"),
            ("b!", "ы"),
            ("b1", "ы"),
            ("61", "61"),
            ("a61", "аы"),
            ("m61", "мы"),
            ("e\"", "ё"),
            ("e~", "ё"),
            ("*", "*"),
            ("a*a", "ажа"),
            ("a|oa", "аюа"),
            ("J|", "Л"),
            ("a/\\a", "ала"),
            ("(|)", "ф"),
            ("u_", "ц"),
            ("LLI", "Ш"),
            ("LLI_", "Щ"),
            ("III_", "Щ"),
            ("a~b", "аъ"),
            ("~)to", "это"),
            ("u~", "й"),
            ("I-0", "Ю"),
            ("9|", "я"),
            ("q", "я"),
            ("mb|", "мы"),
            ("b|l", "ыл"),
            ("mo|0", "мою"),
            ("2026", "2026"),
            ("4 mom 10", "4 мом 10"),
        ],
    );
}

#[test]
fn mongolian_mns_5217() {
    check(
        Scheme::MongolianMns,
        &[
            ("Avarga, khalbaga, aav", "Аварга, халбага, аав"),
            ("Ulaanbaatar", "Улаанбаатар"),
            ("Mongol uls", "Монгол улс"),
            ("tsagaan chuluu", "цагаан чулуу"),
            ("Shine jil", "Шинэ жил"),
            ("ünen", "үнэн"),
            ("yu ya yo ye", "ю я ё е"),
            ("öndör", "өндөр"),
        ],
    );
    assert_eq!(Scheme::MongolianMns.language(), "mn");
    assert!(!Scheme::MongolianMns.is_draft());
}

#[test]
fn ukrainian_national_2010() {
    check(
        Scheme::UkrainianKmu,
        &[
            ("Zghorany", "Згорани"),
            ("Shchastia", "Щастя"),
            ("diakuiu", "дякую"),
            ("yizhak", "їжак"),
            ("Ie", "Є"),
            ("hora", "гора"),
            ("gandzh", "ґандж"),
        ],
    );
    assert_eq!(Scheme::UkrainianKmu.language(), "uk");
    assert_eq!(Scheme::for_language("uk"), Some(Scheme::Ukrainian));
    assert_eq!(Scheme::for_language("mn"), None);
}
