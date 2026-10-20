export const LANGUAGES = ['en', 'ru', 'de', 'es', 'fr', 'it', 'ja', 'pl', 'pt', 'sr-Latn', 'tr', 'zh'] as const;
export type Language = (typeof LANGUAGES)[number];
export type PluralForms = Partial<Record<Intl.LDMLPluralRule, string>>;
export type Entry = string | PluralForms;
export type Dictionary = Record<string, Entry>;
export type Dictionaries = Record<Language, Dictionary>;
export type Vars = Record<string, string | number>;

const PLACEHOLDER = /\{(\w+)\}/g;
const pluralRules = new Map<Language, Intl.PluralRules>();
const numberFormats = new Map<Language, Intl.NumberFormat>();

export function isLanguage(value: unknown): value is Language {
  return typeof value === 'string' && (LANGUAGES as readonly string[]).includes(value);
}

export function pluralCategory(language: Language, count: number): Intl.LDMLPluralRule {
  let rules = pluralRules.get(language);
  if (!rules) {
    rules = new Intl.PluralRules(language);
    pluralRules.set(language, rules);
  }
  return rules.select(count);
}

function formatValue(language: Language, value: string | number): string {
  if (typeof value === 'string') {
    return value;
  }
  let format = numberFormats.get(language);
  if (!format) {
    format = new Intl.NumberFormat(language);
    numberFormats.set(language, format);
  }
  return format.format(value);
}

export function interpolate(language: Language, text: string, vars?: Vars): string {
  if (!vars) {
    return text;
  }
  return text.replace(PLACEHOLDER, (whole, name: string) => {
    const value = vars[name];
    return value === undefined ? whole : formatValue(language, value);
  });
}

export function pick(language: Language, entry: Entry, vars?: Vars): string {
  if (typeof entry === 'string') {
    return entry;
  }
  const count = Number(vars?.count ?? 0);
  return entry[pluralCategory(language, count)] ?? entry.other ?? '';
}

export function translate(dictionaries: Dictionaries, language: Language, key: string, vars?: Vars): string {
  const entry = dictionaries[language][key] ?? dictionaries.en[key];
  if (entry === undefined) {
    return key;
  }
  return interpolate(language, pick(language, entry, vars), vars);
}

export function splitRich(text: string): Array<{ text: string } | { slot: string }> {
  const parts: Array<{ text: string } | { slot: string }> = [];
  let last = 0;
  for (const match of text.matchAll(PLACEHOLDER)) {
    const index = match.index ?? 0;
    if (index > last) {
      parts.push({ text: text.slice(last, index) });
    }
    parts.push({ slot: match[1] ?? '' });
    last = index + match[0].length;
  }
  if (last < text.length) {
    parts.push({ text: text.slice(last) });
  }
  return parts;
}
