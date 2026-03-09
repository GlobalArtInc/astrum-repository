import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

export const LANGUAGES = ['en', 'ru'];
const PLACEHOLDER = /\{(\w+)\}/g;
const SOURCE = /\.(ts|tsx)$/;
const TEST = /\.test\.(ts|tsx)$/;

function walk(dir, found = []) {
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) {
      walk(path, found);
    } else {
      found.push(path);
    }
  }
  return found;
}

export function namespaceOf(file, root) {
  const parts = relative(root, file).split(sep);
  const strings = parts.indexOf('strings');
  if (strings < 1) {
    return null;
  }
  if (parts[0] === 'areas') {
    return strings === 2 ? parts[1] : null;
  }
  return strings === 1 ? parts[0] : null;
}

function placeholders(entry) {
  const texts = typeof entry === 'string' ? [entry] : Object.values(entry);
  const names = new Set();
  for (const text of texts) {
    for (const match of String(text).matchAll(PLACEHOLDER)) {
      names.add(match[1]);
    }
  }
  return [...names].sort().join(',');
}

export function checkDictionaries(dictionaries) {
  const problems = [];
  for (const [namespace, byLanguage] of Object.entries(dictionaries)) {
    const english = byLanguage.en ?? {};
    for (const language of LANGUAGES) {
      const dictionary = byLanguage[language];
      if (!dictionary) {
        problems.push(`${namespace}: ${language}.json is missing`);
        continue;
      }
      const categories = new Intl.PluralRules(language).resolvedOptions().pluralCategories;
      for (const [key, entry] of Object.entries(dictionary)) {
        if (!key.startsWith(`${namespace}.`)) {
          problems.push(`${namespace}/${language}: "${key}" does not start with "${namespace}."`);
        }
        if (key.split('.').length < 3) {
          problems.push(`${namespace}/${language}: "${key}" is not <area>.<page>.<name>`);
        }
        if (typeof entry === 'string') {
          if (entry.trim() === '') {
            problems.push(`${namespace}/${language}: "${key}" is empty`);
          }
        } else if (entry && typeof entry === 'object' && !Array.isArray(entry)) {
          for (const category of categories) {
            if (typeof entry[category] !== 'string' || entry[category].trim() === '') {
              problems.push(`${namespace}/${language}: "${key}" lacks the plural form "${category}"`);
            }
          }
          for (const category of Object.keys(entry)) {
            if (!categories.includes(category)) {
              problems.push(
                `${namespace}/${language}: "${key}" has the form "${category}" ${language} does not use`,
              );
            }
          }
        } else {
          problems.push(`${namespace}/${language}: "${key}" must be a text or plural forms`);
        }
        if (language !== 'en') {
          if (!(key in english)) {
            problems.push(`${namespace}/${language}: "${key}" is not in en.json`);
          } else if (placeholders(english[key]) !== placeholders(entry)) {
            problems.push(`${namespace}/${language}: "${key}" has other placeholders than in en.json`);
          } else if (typeof english[key] !== typeof entry) {
            problems.push(`${namespace}/${language}: "${key}" is plural in one language only`);
          }
        }
      }
      if (language !== 'en') {
        for (const key of Object.keys(english)) {
          if (!(key in dictionary)) {
            problems.push(`${namespace}/${language}: "${key}" is missing`);
          }
        }
      }
    }
  }
  return problems;
}

export function usedKeys(text, namespaces) {
  const pattern = new RegExp(
    `['"\`]((?:${namespaces.join('|')})\\.[A-Za-z0-9_]+(?:\\.[A-Za-z0-9_]+)+)['"\`]`,
    'g',
  );
  return [...text.matchAll(pattern)].map((match) => match[1]);
}

export function loadDictionaries(root) {
  const dictionaries = {};
  for (const file of walk(root).filter((path) => /[/\\]strings[/\\](en|ru)\.json$/.test(path))) {
    const namespace = namespaceOf(file, root);
    const language = file.endsWith('en.json') ? 'en' : 'ru';
    if (!namespace) {
      throw new Error(`${relative(root, file)} is not in <area>/strings or areas/<area>/strings`);
    }
    dictionaries[namespace] ??= {};
    dictionaries[namespace][language] = JSON.parse(readFileSync(file, 'utf8'));
  }
  return dictionaries;
}

export function check(root) {
  const dictionaries = loadDictionaries(root);
  const problems = checkDictionaries(dictionaries);
  const namespaces = Object.keys(dictionaries);
  const known = new Set(
    Object.values(dictionaries).flatMap((byLanguage) => Object.keys(byLanguage.en ?? {})),
  );
  const used = new Set();
  for (const file of walk(root).filter((path) => SOURCE.test(path) && !TEST.test(path))) {
    for (const key of usedKeys(readFileSync(file, 'utf8'), namespaces)) {
      used.add(key);
      if (!known.has(key)) {
        problems.push(`${relative(root, file)}: "${key}" is not in any dictionary`);
      }
    }
  }
  const unused = [...known].filter((key) => !used.has(key));
  return { problems, unused, count: known.size };
}

function main() {
  const root = fileURLToPath(new URL('../src', import.meta.url));
  const { problems, unused, count } = check(root);
  for (const key of unused) {
    console.warn(`unused: ${key}`);
  }
  if (problems.length > 0) {
    for (const problem of problems) {
      console.error(problem);
    }
    console.error(`${problems.length} problem(s) in the UI texts`);
    process.exit(1);
  }
  console.log(`${count} keys in ${LANGUAGES.join(' and ')} match and every used key exists`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
