export type Theme = 'system' | 'light' | 'dark';

export function getTheme(): Theme {
  try {
    const t = localStorage.getItem('theme');
    if (t === 'light' || t === 'dark') return t;
  } catch {
    // ignore
  }
  return 'system';
}

export function applyTheme(theme: Theme) {
  const root = document.documentElement;
  if (theme === 'system') delete root.dataset.theme;
  else root.dataset.theme = theme;
  try {
    localStorage.setItem('theme', theme);
  } catch {
    // ignore
  }
}
