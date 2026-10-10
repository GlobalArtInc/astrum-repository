import React from 'react';

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

describe('UIStrings in Russian', () => {
  let strings;

  beforeAll(() => {
    window.AS_I18N = {language: 'ru', dictionaries: {'react-coreui': ru}};
    jest.isolateModules(() => {
      strings = require('./UIStrings').default;
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

  it('puts the elements of every translated element around its translation', () => {
    const broken = Object.keys(ru).filter((key) => {
      const value = valueAt(strings, key);
      return React.isValidElement(value) && textOf(value.props.children) !== ru[key].replace(/<\/?\d+\/?>/g, '');
    });

    expect(broken).toEqual([]);
  });

  it('replaces the texts computed by getters', () => {
    const acknowledgement = strings.REALMS.LOCAL_REALM_REMOVAL_MODAL.ACKNOWLEDGEMENT;

    expect(acknowledgement.SUBLABEL).toBe(ru['REALMS.LOCAL_REALM_REMOVAL_MODAL.ACKNOWLEDGEMENT.SUBLABEL']);
    expect(acknowledgement.SUBLABEL).toContain(acknowledgement.STRING);
  });
});
