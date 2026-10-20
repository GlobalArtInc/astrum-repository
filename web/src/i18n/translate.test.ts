import { describe, expect, it } from 'vitest';
import { collect } from './dictionaries';
import { detectLanguage, LANGUAGE_COOKIE, readCookie, rememberLanguage } from './language';
import { type Dictionaries, interpolate, pluralCategory, splitRich, translate } from './translate';

const dictionaries = {
  en: {
    'x.page.hello': 'Hello, {name}',
    'x.page.files': { one: '{count} file', other: '{count} files' },
    'x.page.onlyEnglish': 'Only English',
  },
  ru: {
    'x.page.hello': 'Привет, {name}',
    'x.page.files': {
      one: '{count} файл',
      few: '{count} файла',
      many: '{count} файлов',
      other: '{count} файла',
    },
  },
} as unknown as Dictionaries;

describe('translation', () => {
  it('fills placeholders', () => {
    expect(translate(dictionaries, 'en', 'x.page.hello', { name: 'Jane' })).toBe('Hello, Jane');
    expect(translate(dictionaries, 'ru', 'x.page.hello', { name: 'Яна' })).toBe('Привет, Яна');
    expect(interpolate('en', 'keeps {unknown}', {})).toBe('keeps {unknown}');
  });

  it('picks plural forms of each language', () => {
    expect(translate(dictionaries, 'en', 'x.page.files', { count: 1 })).toBe('1 file');
    expect(translate(dictionaries, 'en', 'x.page.files', { count: 1200 })).toBe('1,200 files');
    expect(translate(dictionaries, 'ru', 'x.page.files', { count: 1 })).toBe('1 файл');
    expect(translate(dictionaries, 'ru', 'x.page.files', { count: 3 })).toBe('3 файла');
    expect(translate(dictionaries, 'ru', 'x.page.files', { count: 5 })).toBe('5 файлов');
    expect(translate(dictionaries, 'ru', 'x.page.files', { count: 21 })).toBe('21 файл');
    expect(pluralCategory('ru', 12)).toBe('many');
  });

  it('falls back to English, then to the key', () => {
    expect(translate(dictionaries, 'ru', 'x.page.onlyEnglish')).toBe('Only English');
    expect(translate(dictionaries, 'ru', 'x.page.missing')).toBe('x.page.missing');
  });

  it('splits rich texts into slots', () => {
    expect(splitRich('Nothing at {address}.')).toEqual([
      { text: 'Nothing at ' },
      { slot: 'address' },
      { text: '.' },
    ]);
    expect(splitRich('{a}{b}')).toEqual([{ slot: 'a' }, { slot: 'b' }]);
  });

  it('merges dictionaries by language', () => {
    const merged = collect({
      '../a/strings/en.json': { 'a.b.c': 'A' },
      '../b/strings/en.json': { 'b.c.d': 'B' },
      '../a/strings/ru.json': { 'a.b.c': 'А' },
      '../a/strings/de.json': { 'a.b.c': 'X' },
    });
    expect(merged.en).toEqual({ 'a.b.c': 'A', 'b.c.d': 'B' });
    expect(merged.ru).toEqual({ 'a.b.c': 'А' });
  });
});

describe('the language', () => {
  it('comes from the cookie, then from the browser', () => {
    expect(detectLanguage(`${LANGUAGE_COOKIE}=ru`, ['en-US'])).toBe('ru');
    expect(detectLanguage('other=1', ['ru-RU', 'en'])).toBe('ru');
    expect(detectLanguage('', ['sv-SE', 'en-GB'])).toBe('en');
    expect(detectLanguage(`${LANGUAGE_COOKIE}=sv`, ['fr'])).toBe('fr');
    expect(detectLanguage('', ['sr-Latn'])).toBe('sr-Latn');
  });

  it('is remembered in its cookie', () => {
    rememberLanguage('ru');
    expect(readCookie(LANGUAGE_COOKIE)).toBe('ru');
    expect(detectLanguage(document.cookie, ['en'])).toBe('ru');
  });
});
