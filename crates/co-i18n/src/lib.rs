//! Translations shared by printed lists (Rust) and the desktop UI.
//!
//! Message files live in `locales/<lang>/*.ftl` at the repository root and
//! are embedded at build time. Missing messages fall back to English, then to
//! the message id itself, so a partly translated language never breaks a list.

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use unic_langid::LanguageIdentifier;

/// Languages shipped with the program, English first.
pub const SUPPORTED: &[&str] = &["en", "fr", "ar", "sw", "pt", "es"];

macro_rules! ftl {
    ($lang:literal) => {
        include_str!(concat!("../../../locales/", $lang, "/lists.ftl"))
    };
}

fn sources(lang: &str) -> Option<&'static str> {
    Some(match lang {
        "en" => ftl!("en"),
        "fr" => ftl!("fr"),
        "ar" => ftl!("ar"),
        "sw" => ftl!("sw"),
        "pt" => ftl!("pt"),
        "es" => ftl!("es"),
        _ => return None,
    })
}

/// Whether `lang` is written right to left.
pub fn is_rtl(lang: &str) -> bool {
    matches!(primary(lang), "ar" | "fa" | "he" | "ur")
}

fn primary(lang: &str) -> &str {
    lang.split(['-', '_']).next().unwrap_or(lang)
}

fn bundle(lang: &str, source: &'static str) -> FluentBundle<FluentResource> {
    let id: LanguageIdentifier = lang.parse().unwrap_or_default();
    let mut bundle = FluentBundle::new_concurrent(vec![id]);
    // Lists are rendered as plain text/HTML; bidi isolation marks would leak into CSV/TRF.
    bundle.set_use_isolating(false);
    let resource = FluentResource::try_new(source.to_owned()).unwrap_or_else(|(res, _)| res);
    let _ = bundle.add_resource(resource);
    bundle
}

pub struct Translator {
    lang: String,
    primary: Option<FluentBundle<FluentResource>>,
    english: FluentBundle<FluentResource>,
}

impl Translator {
    pub fn new(lang: &str) -> Self {
        let lang = primary(lang).to_owned();
        let primary = (lang != "en")
            .then(|| sources(&lang).map(|s| bundle(&lang, s)))
            .flatten();
        Self {
            english: bundle("en", ftl!("en")),
            primary,
            lang,
        }
    }

    pub fn lang(&self) -> &str {
        &self.lang
    }

    pub fn is_rtl(&self) -> bool {
        is_rtl(&self.lang)
    }

    pub fn t(&self, id: &str) -> String {
        self.format(id, None)
    }

    pub fn t_args(&self, id: &str, args: &[(&str, FluentValue<'_>)]) -> String {
        let mut fa = FluentArgs::new();
        for (k, v) in args {
            fa.set(*k, v.clone());
        }
        self.format(id, Some(&fa))
    }

    fn format(&self, id: &str, args: Option<&FluentArgs>) -> String {
        for bundle in self.primary.iter().chain(std::iter::once(&self.english)) {
            if let Some(pattern) = bundle.get_message(id).and_then(|m| m.value()) {
                let mut errors = Vec::new();
                return bundle
                    .format_pattern(pattern, args, &mut errors)
                    .into_owned();
            }
        }
        id.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_and_falls_back_to_the_id() {
        let t = Translator::new("fr");
        assert_eq!(t.t("col-name"), "Nom");
        assert_eq!(t.t("no-such-message"), "no-such-message");
        assert_eq!(
            t.t_args("list-pairings", &[("round", 3.into())]),
            "Appariements – ronde 3"
        );
        assert_eq!(Translator::new("de").t("col-name"), "Name");
    }

    /// Message ids and the variables each message uses, from FTL source.
    fn catalogue(
        src: &str,
    ) -> std::collections::BTreeMap<String, std::collections::BTreeSet<String>> {
        let mut out = std::collections::BTreeMap::new();
        let mut current: Option<String> = None;
        for line in src.lines() {
            if let Some((id, _)) = line.split_once(" = ").filter(|(id, _)| {
                !id.is_empty()
                    && id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            }) {
                current = Some(id.to_string());
                out.insert(id.to_string(), Default::default());
            }
            if let Some(id) = &current {
                let vars: &mut std::collections::BTreeSet<String> =
                    out.get_mut(id).expect("inserted");
                let mut rest = line;
                while let Some(i) = rest.find("{ $") {
                    let after = &rest[i + 3..];
                    let end = after
                        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                        .unwrap_or(after.len());
                    vars.insert(after[..end].to_string());
                    rest = &after[end..];
                }
            }
        }
        out
    }

    #[test]
    fn every_language_matches_english() {
        macro_rules! app {
            ($l:literal) => {
                include_str!(concat!("../../../locales/", $l, "/app.ftl"))
            };
        }
        let english = [catalogue(ftl!("en")), catalogue(app!("en"))];
        let others = [
            ("fr", ftl!("fr"), app!("fr")),
            ("ar", ftl!("ar"), app!("ar")),
            ("sw", ftl!("sw"), app!("sw")),
            ("pt", ftl!("pt"), app!("pt")),
            ("es", ftl!("es"), app!("es")),
        ];
        for (lang, lists, app) in others {
            for (file, (en, src)) in ["lists.ftl", "app.ftl"]
                .iter()
                .zip(english.iter().zip([lists, app]))
            {
                let theirs = catalogue(src);
                let missing: Vec<_> = en.keys().filter(|k| !theirs.contains_key(*k)).collect();
                let extra: Vec<_> = theirs.keys().filter(|k| !en.contains_key(*k)).collect();
                assert!(
                    missing.is_empty() && extra.is_empty(),
                    "{lang}/{file}: missing {missing:?}, extra {extra:?}"
                );
                for (k, vars) in en {
                    assert_eq!(&theirs[k], vars, "{lang}/{file}: variables of {k}");
                }
                // Every message must parse.
                FluentResource::try_new(src.to_owned())
                    .unwrap_or_else(|(_, e)| panic!("{lang}/{file}: {e:?}"));
            }
        }
    }

    #[test]
    fn detects_rtl() {
        assert!(is_rtl("ar"));
        assert!(is_rtl("ar-EG"));
        assert!(!is_rtl("sw"));
    }
}
