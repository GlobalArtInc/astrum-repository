import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { check, checkDictionaries, namespaceOf, usedKeys } from './i18n-check.mjs';

describe('the UI text check', () => {
  it('accepts matching dictionaries', () => {
    expect(
      checkDictionaries({
        demo: {
          en: {
            'demo.page.title': 'Title',
            'demo.page.files': { one: '{count} file', other: '{count} files' },
          },
          ru: {
            'demo.page.title': 'Заголовок',
            'demo.page.files': {
              one: '{count} файл',
              few: '{count} файла',
              many: '{count} файлов',
              other: '{count} файла',
            },
          },
        },
      }),
    ).toEqual([]);
  });

  it('finds missing keys, forms, placeholders and prefixes', () => {
    const problems = checkDictionaries({
      demo: {
        en: {
          'demo.page.title': 'Hello {name}',
          'demo.page.files': { one: '{count} file', other: '{count} files' },
          'other.page.x': 'X',
          'demo.short': 'Short',
          'demo.page.empty': ' ',
        },
        ru: {
          'demo.page.title': 'Привет',
          'demo.page.files': { one: '{count} файл', other: '{count} файла' },
          'demo.page.extra': 'Лишнее',
        },
      },
    });
    expect(problems).toEqual(
      expect.arrayContaining([
        'demo/en: "other.page.x" does not start with "demo."',
        'demo/en: "demo.short" is not <area>.<page>.<name>',
        'demo/en: "demo.page.empty" is empty',
        'demo/ru: "demo.page.title" has other placeholders than in en.json',
        'demo/ru: "demo.page.files" lacks the plural form "few"',
        'demo/ru: "demo.page.files" lacks the plural form "many"',
        'demo/ru: "demo.page.extra" is not in en.json',
        'demo/ru: "other.page.x" is missing',
      ]),
    );
  });

  it('finds keys used in the code', () => {
    const dynamic = ['$', '{name}'].join('');
    const text = `t('shell.menu.open'); const x = { label: "dashboard.menu.label" }; t(\`ui.table.${dynamic}\`); 'shell.only'`;
    expect(usedKeys(text, ['shell', 'dashboard', 'ui'])).toEqual(['shell.menu.open', 'dashboard.menu.label']);
  });

  it('names the owner of a dictionary', () => {
    expect(namespaceOf('/src/areas/browse/strings/en.json', '/src')).toBe('browse');
    expect(namespaceOf('/src/shell/strings/ru.json', '/src')).toBe('shell');
    expect(namespaceOf('/src/areas/browse/pages/strings/en.json', '/src')).toBeNull();
  });

  it('passes on the real dictionaries', () => {
    const { problems, count } = check(resolve('src'));
    expect(problems).toEqual([]);
    expect(count).toBeGreaterThan(100);
  });
});
