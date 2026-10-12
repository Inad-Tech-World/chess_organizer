// UI translations. Messages live in ../locales/<lang>/*.ftl at the repository
// root, shared with the Rust list renderer (crates/co-i18n). Missing messages
// fall back to English, then to the message id.

import { FluentBundle, FluentResource, type FluentVariable } from '@fluent/bundle';

const sources = import.meta.glob('../../../../locales/*/*.ftl', {
  query: '?raw',
  import: 'default',
  eager: true
}) as Record<string, string>;

export const LANGUAGES = [
  { code: 'en', name: 'English' },
  { code: 'fr', name: 'Français' },
  { code: 'ar', name: 'العربية' },
  { code: 'sw', name: 'Kiswahili' },
  { code: 'pt', name: 'Português' },
  { code: 'es', name: 'Español' }
];

const RTL = new Set(['ar', 'fa', 'he', 'ur']);

export function isRtl(lang: string): boolean {
  return RTL.has(lang.split('-')[0]);
}

function bundleFor(lang: string): FluentBundle {
  const bundle = new FluentBundle(lang, { useIsolating: false });
  for (const [path, text] of Object.entries(sources)) {
    if (path.includes(`/locales/${lang}/`)) bundle.addResource(new FluentResource(text));
  }
  return bundle;
}

const english = bundleFor('en');
let current = english;

export const i18n = $state({ lang: 'en' });

export function setLang(lang: string) {
  current = lang === 'en' ? english : bundleFor(lang);
  i18n.lang = lang;
  document.documentElement.lang = lang;
  document.documentElement.dir = isRtl(lang) ? 'rtl' : 'ltr';
  try {
    localStorage.setItem('lang', lang);
  } catch {
    // Storage unavailable: the choice lasts for this session only.
  }
}

export function initLang() {
  let saved: string | null = null;
  try {
    saved = localStorage.getItem('lang');
  } catch {
    saved = null;
  }
  setLang(saved && LANGUAGES.some((l) => l.code === saved) ? saved : 'en');
}

/** Translates `id`. Reading `i18n.lang` makes callers re-render on change. */
export function t(id: string, args?: Record<string, FluentVariable>): string {
  void i18n.lang;
  for (const bundle of [current, english]) {
    const msg = bundle.getMessage(id);
    if (msg?.value) return bundle.formatPattern(msg.value, args);
  }
  return id;
}
