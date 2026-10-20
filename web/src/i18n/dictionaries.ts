import { type Dictionaries, type Dictionary, isLanguage, LANGUAGES } from './translate';

const files = import.meta.glob<Dictionary>('../**/strings/*.json', { eager: true, import: 'default' });

export function collect(modules: Record<string, Dictionary>): Dictionaries {
  const dictionaries = Object.fromEntries(LANGUAGES.map((language) => [language, {}])) as Dictionaries;
  for (const [path, dictionary] of Object.entries(modules)) {
    const language = path
      .split('/')
      .pop()
      ?.replace(/\.json$/, '');
    if (isLanguage(language)) {
      Object.assign(dictionaries[language], dictionary);
    }
  }
  return dictionaries;
}

export const dictionaries = collect(files);
