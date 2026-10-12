import React from 'react';
import fs from 'fs';
import path from 'path';

const ru = require('../../../main/resources/astrum-i18n/react-coreui/ru.json');

function valueAt(strings, key) {
  return key.split('.').reduce((node, part) => (node == null ? undefined : node[part]), strings);
}

function textOf(children) {
  return React.Children.toArray(children).map((child) => {
    if (React.isValidElement(child)) {
      return textOf(child.props.children);
    }
    return typeof child === 'string' || typeof child === 'number' ? String(child) : '';
  }).join('');
}

const spreadModules = (() => {
  const source = fs.readFileSync(path.join(__dirname, './UIStrings.jsx'), 'utf8');
  const imports = Object.fromEntries([...source.matchAll(/import (\w+) from ['"](\.[^'"]+)['"]/g)].map(([, name, from]) => [name, from]));
  return [...source.matchAll(/\.\.\.(\w+Strings)\b/g)].map(([, name]) => imports[name]).filter(Boolean);
})();

describe('UIStrings in Russian', () => {
  let strings;
  let modules;

  beforeAll(() => {
    window.AS_I18N = {language: 'ru', dictionaries: {'react-coreui': ru}};
    jest.isolateModules(() => {
      strings = require('./UIStrings').default;
      modules = Object.fromEntries(spreadModules.map((from) => [from, require(from).default]));
    });
  });

  afterAll(() => {
    delete window.AS_I18N;
  });

  it('applies every translated text', () => {
    const untranslated = Object.keys(ru).filter((key) => {
      const value = valueAt(strings, key);
      return typeof value === 'string' && value !== ru[key];
    });

    expect(untranslated).toEqual([]);
  });

  it('translates the strings modules in place, for the components that import them directly', () => {
    const untranslated = Object.entries(modules).flatMap(([from, module]) => Object.keys(ru).filter((key) => {
      const value = valueAt(module, key);
      return typeof value === 'string' && value !== ru[key];
    }).map((key) => `${from}: ${key}`));

    expect(spreadModules.length).toBeGreaterThan(0);
    expect(untranslated).toEqual([]);
  });

  it('puts the elements of every translated element around its translation', () => {
    const broken = Object.keys(ru).filter((key) => {
      const value = valueAt(strings, key);
      return React.isValidElement(value) && textOf(value.props.children) !== ru[key].replace(/<\/?\d+\/?>/g, '');
    });

    expect(broken).toEqual([]);
  });
});
