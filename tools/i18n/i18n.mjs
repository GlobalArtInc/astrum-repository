#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const CONFIG = JSON.parse(fs.readFileSync(path.join(ROOT, 'tools/i18n/dictionaries.json'), 'utf8'));
const PLURAL_FORMS = ['zero', 'one', 'two', 'few', 'many', 'other'];

const usage = `Usage: node tools/i18n/i18n.mjs <command>

  check                fail when a translation has a key the source lacks, other plural forms
                       than its language needs, or different placeholders than the source
  status               print how much of each dictionary is translated
  pull                 fetch the translations from Ownlate into <dir>/<lang>.json
                       (needs ${CONFIG.tokenEnv} and ${CONFIG.projectEnv})
  import <lang> <file> write <dir>/<lang>.json from a JSON file exported from Ownlate
  catalogue <lang>     print every entry of <lang> under its Ownlate key, for seeding a project`;

const DICTIONARIES = CONFIG.dictionaries.flatMap(webDictionaries);

const sources = new Map();

function webDictionaries(dictionary) {
  const found = [];
  const walk = (dir) => {
    const children = fs.readdirSync(path.join(ROOT, dir), { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name)
      .sort();
    for (const name of children) {
      const child = path.posix.join(dir, name);
      if (name === 'strings' && fs.existsSync(path.join(ROOT, child, `${CONFIG.source}.json`))) {
        found.push({ dir: child, prefix: dictionary.prefix });
      }
      else {
        walk(child);
      }
    }
  };
  walk(dictionary.web);
  return found;
}

function sourceOf(dictionary) {
  if (!sources.has(dictionary.dir)) {
    sources.set(dictionary.dir, read(dictionary, CONFIG.source));
  }
  return sources.get(dictionary.dir);
}

function sorted(entries) {
  return Object.fromEntries(Object.keys(entries).sort().map((key) => [key, entries[key]]));
}

function serialize(entries) {
  return `${JSON.stringify(sorted(entries), null, 2)}\n`;
}

// A translation keeps the layout of its en.json: the same key order and the same blank lines between groups.
function serializeTranslation(dictionary, entries) {
  const groups = groupStarts(dictionary);
  const lines = JSON.stringify(entries, null, 2).split('\n').map((line) => {
    const match = /^ {2}("(?:[^"\\]|\\.)*"):/.exec(line);
    return match && groups.has(JSON.parse(match[1])) ? `\n${line}` : line;
  });
  return `${lines.join('\n')}\n`;
}

function groupStarts(dictionary) {
  const lines = fs.readFileSync(file(dictionary, CONFIG.source), 'utf8').split('\n');
  const keys = new Set();
  lines.forEach((line, index) => {
    const match = /^\s*("(?:[^"\\]|\\.)*")\s*:/.exec(line);
    if (match && index > 0 && lines[index - 1].trim() === '') {
      keys.add(JSON.parse(match[1]));
    }
  });
  return keys;
}

function file(dictionary, language) {
  return path.join(ROOT, dictionary.dir, `${language}.json`);
}

function read(dictionary, language) {
  const name = file(dictionary, language);
  return fs.existsSync(name) ? JSON.parse(fs.readFileSync(name, 'utf8')) : null;
}

function languages(dictionary) {
  const dir = path.join(ROOT, dictionary.dir);
  if (!fs.existsSync(dir)) {
    return [];
  }
  return fs.readdirSync(dir)
    .filter((name) => name.endsWith('.json'))
    .map((name) => name.slice(0, -'.json'.length))
    .filter((language) => language !== CONFIG.source)
    .sort();
}

function pluralForms(languageList) {
  const needed = new Set(languageList.flatMap((language) =>
    new Intl.PluralRules(language).resolvedOptions().pluralCategories));
  return PLURAL_FORMS.filter((form) => needed.has(form));
}

function isPlural(entry) {
  return Boolean(entry) && typeof entry === 'object' && !Array.isArray(entry);
}

// Every text of a dictionary in a language under its Ownlate key. A plural entry becomes one key per
// form, <key>#<form>; the source offers every form its translations need, filled with its "other" text where English
// has no such form.
function segments(dictionary, language) {
  const isSource = language === CONFIG.source;
  const messages = isSource ? sourceOf(dictionary) : read(dictionary, language) || {};
  const sourceForms = pluralForms([CONFIG.source, ...languages(dictionary)]);
  const entries = {};
  for (const [key, text] of Object.entries(messages)) {
    const name = `${dictionary.prefix}.${key}`;
    if (typeof text === 'string') {
      entries[name] = text;
    }
    else if (isPlural(text)) {
      for (const form of isSource ? sourceForms : PLURAL_FORMS.filter((form) => form in text)) {
        const value = text[form] ?? (isSource ? text.other : undefined);
        if (typeof value === 'string') {
          entries[`${name}#${form}`] = value;
        }
      }
    }
  }
  return entries;
}

function texts(entry) {
  return typeof entry === 'string' ? [entry] : Object.values(entry);
}

function placeholders(entry) {
  const found = texts(entry).flatMap((text) => String(text).match(/\{\w+\}/g) || []);
  return [...new Set(found)].sort().join(' ');
}

function mismatch(language, source, entry) {
  if (source === undefined) {
    return 'is not in the source';
  }
  if (typeof source === 'string' && typeof entry !== 'string') {
    return 'is not a string';
  }
  if (typeof source !== 'string') {
    const forms = pluralForms([language]);
    if (!isPlural(entry)) {
      return 'is not plural forms';
    }
    if (Object.keys(entry).sort().join(' ') !== [...forms].sort().join(' ')
      || !Object.values(entry).every((text) => typeof text === 'string' && text !== '')) {
      return `has the plural forms [${Object.keys(entry).join(' ')}], ${language} needs [${forms.join(' ')}]`;
    }
  }
  if (placeholders(entry) !== placeholders(source)) {
    return `has placeholders [${placeholders(entry)}], the source [${placeholders(source)}]`;
  }
  return null;
}

function check() {
  const problems = [];
  for (const dictionary of DICTIONARIES) {
    const source = sourceOf(dictionary);
    for (const language of languages(dictionary)) {
      for (const [key, entry] of Object.entries(read(dictionary, language))) {
        const problem = mismatch(language, source[key], entry);
        if (problem) {
          problems.push(`${dictionary.dir}/${language}.json: ${key} ${problem}`);
        }
      }
    }
  }
  problems.forEach((problem) => console.error(problem));
  if (problems.length > 0) {
    process.exit(1);
  }
  console.log('dictionaries are consistent');
}

function status() {
  for (const dictionary of DICTIONARIES) {
    const source = sourceOf(dictionary);
    const total = Object.keys(source).length;
    for (const language of languages(dictionary)) {
      const translations = read(dictionary, language);
      const done = Object.keys(source).filter((key) => translations[key]).length;
      console.log(`${dictionary.dir} ${language}: ${done}/${total}`);
    }
  }
}

function flatten(node, prefix, into) {
  for (const [key, value] of Object.entries(node)) {
    const name = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === 'object') {
      flatten(value, name, into);
    }
    else if (typeof value === 'string' && value !== '') {
      into[name] = value;
    }
  }
  return into;
}

async function pull() {
  const token = process.env[CONFIG.tokenEnv];
  const project = process.env[CONFIG.projectEnv];
  if (!token || !project) {
    console.error(`${CONFIG.tokenEnv} and ${CONFIG.projectEnv} must be set`);
    process.exit(2);
  }
  const url = `${CONFIG.url.replace(/\/$/, '')}/public/v1/segments/translations-map?projectId=${encodeURIComponent(project)}`;
  const response = await fetch(url, { headers: { Authorization: `Bearer ${token}`, Accept: 'application/json' } });
  const body = await response.text();
  if (response.status !== 200) {
    console.error(`${url} answered ${response.status}: ${body.slice(0, 300)}`);
    process.exit(1);
  }
  const byLanguage = {};
  for (const files of Object.values(JSON.parse(body))) {
    for (const [language, messages] of Object.entries(files)) {
      Object.assign(byLanguage[language] ??= {}, flatten(messages, '', {}));
    }
  }
  for (const [language, messages] of Object.entries(byLanguage)) {
    if (language !== CONFIG.source && !language.startsWith(`${CONFIG.source}_`)) {
      distribute(language, messages);
    }
  }
}

// A plural entry is taken only with every form its language needs; otherwise the page shows the English text.
function entriesOf(dictionary, language, messages) {
  const entries = {};
  for (const [key, text] of Object.entries(sourceOf(dictionary))) {
    const name = `${dictionary.prefix}.${key}`;
    if (typeof text === 'string') {
      if (messages[name]) {
        entries[key] = messages[name];
      }
      continue;
    }
    const forms = pluralForms([language]);
    if (forms.every((form) => messages[`${name}#${form}`])) {
      entries[key] = Object.fromEntries(forms.map((form) => [form, messages[`${name}#${form}`]]));
    }
  }
  return entries;
}

function distribute(language, messages) {
  for (const dictionary of DICTIONARIES) {
    const entries = entriesOf(dictionary, language, messages);
    const name = file(dictionary, language);
    if (Object.keys(entries).length === 0) {
      // an empty dictionary would still offer the language in the UI
      if (fs.existsSync(name)) {
        fs.unlinkSync(name);
        console.log(`removed ${dictionary.dir}/${language}.json (no translations)`);
      }
      continue;
    }
    fs.writeFileSync(name, serializeTranslation(dictionary, entries));
    console.log(`wrote ${dictionary.dir}/${language}.json (${Object.keys(entries).length} entries)`);
  }
}

function importFile(language, name) {
  if (!language || !name) {
    console.error(usage);
    process.exit(2);
  }
  distribute(language, flatten(JSON.parse(fs.readFileSync(name, 'utf8')), '', {}));
}

function catalogue(language) {
  const entries = {};
  for (const dictionary of DICTIONARIES) {
    Object.assign(entries, segments(dictionary, language));
  }
  process.stdout.write(serialize(entries));
}

const [command, argument, extra] = process.argv.slice(2);
switch (command) {
  case 'check': check(); break;
  case 'status': status(); break;
  case 'pull': await pull(); break;
  case 'import': importFile(argument, extra); break;
  case 'catalogue': catalogue(argument || CONFIG.source); break;
  default: console.error(usage); process.exit(command ? 2 : 0);
}
