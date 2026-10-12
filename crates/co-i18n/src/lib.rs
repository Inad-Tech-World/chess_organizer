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
    fn falls_back_to_english_then_id() {
        let t = Translator::new("fr");
        assert_eq!(t.t("col-name"), "Name");
        assert_eq!(t.t("no-such-message"), "no-such-message");
        assert_eq!(
            t.t_args("list-pairings", &[("round", 3.into())]),
            "Pairings – round 3"
        );
    }

    #[test]
    fn detects_rtl() {
        assert!(is_rtl("ar"));
        assert!(is_rtl("ar-EG"));
        assert!(!is_rtl("sw"));
    }
}
