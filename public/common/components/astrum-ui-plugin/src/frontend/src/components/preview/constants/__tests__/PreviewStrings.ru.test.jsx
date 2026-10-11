import React from 'react';

const en = require('../../../../../../main/resources/astrum-i18n/react-ui/en.json');
const ru = require('../../../../../../main/resources/astrum-i18n/react-ui/ru.json');

const PREFIX = 'PREVIEW.';

function textOf(children) {
  return React.Children.toArray(children).map((child) => {
    if (React.isValidElement(child)) {
      return textOf(child.props.children);
    }
    return typeof child === 'string' || typeof child === 'number' ? String(child) : '';
  }).join('');
}

function render(value) {
  const shown = typeof value === 'function' ? value('{0}', '{1}', '{2}', '{3}', '{4}', '{5}') : value;
  return React.isValidElement(shown) ? textOf(shown.props.children) : shown;
}

/**
 * Loads the strings modules of Astrum One with the given translations and returns, for a key, the text the
 * components would show (with `{0}`... for the arguments of a template).
 */
function loadWith(translations) {
  const recorded = [];
  window.AS_I18N = {language: 'ru', dictionaries: {'react-ui': translations}};
  jest.isolateModules(() => {
    const actual = jest.requireActual('../../../../interface/localize').default;
    jest.doMock('../../../../interface/localize', () => ({
      __esModule: true,
      default: (dictionary, strings, scope) => {
        recorded.push({strings, scope: scope || ''});
        return actual(dictionary, strings, scope);
      }
    }));
    require('../PreviewStrings');
  });
  return (key) => {
    const owner = [...recorded].sort((a, b) => b.scope.length - a.scope.length)
        .find(({scope}) => scope && key.startsWith(`${scope}.`));
    if (!owner) {
      return undefined;
    }
    const value = key.slice(owner.scope.length + 1).split('.')
        .reduce((node, part) => (node == null ? undefined : node[part]), owner.strings);
    return render(value);
  };
}

describe('Astrum One strings in Russian', () => {
  afterEach(() => {
    delete window.AS_I18N;
    jest.dontMock('../../../../interface/localize');
  });

  it('has a strings module for every text of the dictionary', () => {
    const shown = loadWith({});
    const missing = Object.keys(en).filter((key) => key.startsWith(PREFIX) && shown(key) === undefined);

    expect(missing).toEqual([]);
  });

  it('shows the translation of every text', () => {
    const fake = Object.fromEntries(Object.entries(en).filter(([key]) => key.startsWith(PREFIX))
        .map(([key, text]) => [key, `ru ${text}`]));
    const shown = loadWith(fake);
    const untranslated = Object.keys(fake).filter((key) => shown(key) !== fake[key].replace(/<\/?\d+\/?>/g, ''));

    expect(untranslated).toEqual([]);
  });

  it('applies the shipped Russian texts', () => {
    const shipped = Object.fromEntries(Object.entries(ru).filter(([key]) => key.startsWith(PREFIX)));
    const shown = loadWith(shipped);
    const untranslated = Object.keys(shipped).filter((key) => shown(key) !== shipped[key].replace(/<\/?\d+\/?>/g, ''));

    expect(untranslated).toEqual([]);
  });
});
