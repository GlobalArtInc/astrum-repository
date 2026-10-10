#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const CONFIG = JSON.parse(fs.readFileSync(path.join(ROOT, 'tools/i18n/dictionaries.json'), 'utf8'));

const usage = `Usage: node tools/i18n/i18n.mjs <command>

  extract              write every dictionary's ${CONFIG.source}.json from the code
  check                fail when a ${CONFIG.source}.json is out of date, or a translation has a key
                       the source lacks or different {0} placeholders or HTML tags than the source
  status               print how much of each dictionary is translated
  pull                 fetch the translations from Ownlate into <dir>/<lang>.json
                       (needs ${CONFIG.tokenEnv} and ${CONFIG.projectEnv})
  import <lang> <file> write <dir>/<lang>.json from a JSON file exported from Ownlate
  catalogue <lang>     print every entry of <lang> under its Ownlate key, for seeding a project`;

const sources = new Map();

function sourceOf(dictionary) {
  if (!sources.has(dictionary.dir)) {
    sources.set(dictionary.dir, readSource(dictionary));
  }
  return sources.get(dictionary.dir).entries;
}

function readSource(dictionary) {
  if (dictionary.extjs) {
    return { entries: extjsStrings(path.join(ROOT, dictionary.extjs)), skipped: [] };
  }
  if (dictionary.react) {
    const { reactStrings } = createRequire(import.meta.url)('./react.cjs');
    return reactStrings(ROOT, dictionary.react, path.basename(dictionary.dir));
  }
  throw new Error(`No source for ${dictionary.dir}`);
}

function extjsStrings(file) {
  let body;
  const Ext = { define: (name, definition) => { body = definition; } };
  vm.runInNewContext(fs.readFileSync(file, 'utf8'), { Ext, AS: {} }, { filename: file });
  const entries = {};
  for (const [key, text] of Object.entries(body.keys || {})) {
    if (translatable(text)) {
      entries[key] = text;
    }
  }
  for (const [bundle, resources] of Object.entries(body.bundles || {})) {
    for (const [key, text] of Object.entries(resources)) {
      if (translatable(text)) {
        entries[`${bundle}/${key}`] = text;
      }
    }
  }
  return entries;
}

function translatable(text) {
  return typeof text === 'string' && text.trim() !== '' && !text.startsWith('@') && /[A-Za-z]/.test(text);
}

function sorted(entries) {
  return Object.fromEntries(Object.keys(entries).sort().map((key) => [key, entries[key]]));
}

function serialize(entries) {
  return `${JSON.stringify(sorted(entries), null, 2)}\n`;
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

function placeholders(text) {
  return (text.match(/\{\d+\}/g) || []).sort().join(' ');
}

function tags(text) {
  const html = (text.match(/<\/?[a-zA-Z][a-zA-Z0-9]*/g) || []).map((tag) => tag.toLowerCase());
  const elements = text.match(/<\/?\d+\/?>/g) || [];
  return [...html, ...elements].sort().join(' ');
}

function extract() {
  for (const dictionary of CONFIG.dictionaries) {
    const name = file(dictionary, CONFIG.source);
    fs.mkdirSync(path.dirname(name), { recursive: true });
    fs.writeFileSync(name, serialize(sourceOf(dictionary)));
    console.log(`wrote ${path.relative(ROOT, name)}`);
    for (const reason of sources.get(dictionary.dir).skipped) {
      console.log(`  not translatable, stays English: ${reason}`);
    }
  }
}

function check() {
  const problems = [];
  for (const dictionary of CONFIG.dictionaries) {
    const source = sourceOf(dictionary);
    const committed = fs.existsSync(file(dictionary, CONFIG.source))
      ? fs.readFileSync(file(dictionary, CONFIG.source), 'utf8')
      : '';
    if (committed !== serialize(source)) {
      problems.push(`${dictionary.dir}/${CONFIG.source}.json is out of date: run node tools/i18n/i18n.mjs extract`);
    }
    for (const language of languages(dictionary)) {
      for (const [key, text] of Object.entries(read(dictionary, language))) {
        const where = `${dictionary.dir}/${language}.json: ${key}`;
        if (!(key in source)) {
          problems.push(`${where} is not in the source`);
        }
        else if (typeof text !== 'string') {
          problems.push(`${where} is not a string`);
        }
        else if (placeholders(text) !== placeholders(source[key])) {
          problems.push(`${where} has placeholders [${placeholders(text)}], the source [${placeholders(source[key])}]`);
        }
        else if (tags(text) !== tags(source[key])) {
          problems.push(`${where} has HTML tags [${tags(text)}], the source [${tags(source[key])}]`);
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
  for (const dictionary of CONFIG.dictionaries) {
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

function distribute(language, messages) {
  for (const dictionary of CONFIG.dictionaries) {
    const source = sourceOf(dictionary);
    const entries = {};
    for (const key of Object.keys(source)) {
      const text = messages[`${dictionary.prefix}.${key}`];
      if (text) {
        entries[key] = text;
      }
    }
    const name = file(dictionary, language);
    if (Object.keys(entries).length === 0) {
      // an empty dictionary would still offer the language in the UI
      if (fs.existsSync(name)) {
        fs.unlinkSync(name);
        console.log(`removed ${dictionary.dir}/${language}.json (no translations)`);
      }
      continue;
    }
    fs.writeFileSync(name, serialize(entries));
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
  for (const dictionary of CONFIG.dictionaries) {
    const messages = language === CONFIG.source ? sourceOf(dictionary) : read(dictionary, language) || {};
    for (const [key, text] of Object.entries(messages)) {
      entries[`${dictionary.prefix}.${key}`] = text;
    }
  }
  process.stdout.write(serialize(entries));
}

const [command, argument, extra] = process.argv.slice(2);
switch (command) {
  case 'extract': extract(); break;
  case 'check': check(); break;
  case 'status': status(); break;
  case 'pull': await pull(); break;
  case 'import': importFile(argument, extra); break;
  case 'catalogue': catalogue(argument || CONFIG.source); break;
  default: console.error(usage); process.exit(command ? 2 : 0);
}
