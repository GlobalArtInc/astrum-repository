import React from 'react';
import {render, screen} from '@testing-library/react';
import '@testing-library/jest-dom';
import localize from './localize';

function withTranslations(dictionaries) {
  window.AS_I18N = {language: 'ru', dictionaries};
}

describe('localize', () => {
  afterEach(() => {
    delete window.AS_I18N;
  });

  it('keeps the English texts when the page has no translations', () => {
    const strings = {TITLE: 'Users'};

    expect(localize('react-test', strings)).toBe(strings);
    expect(strings.TITLE).toBe('Users');
  });

  it('replaces strings by their dotted path, in place', () => {
    withTranslations({'react-test': {'USERS.LIST.TITLE': 'Пользователи', 'LIST.0': 'Первый'}});
    const users = {LIST: {TITLE: 'Users', EMPTY: 'No users'}};
    const strings = {USERS: users, LIST: ['First', 'Second']};

    localize('react-test', strings);

    expect(users.LIST.TITLE).toBe('Пользователи');
    expect(users.LIST.EMPTY).toBe('No users');
    expect(strings.LIST).toEqual(['Первый', 'Second']);
  });

  it('prefixes the keys with the scope', () => {
    withTranslations({'react-test': {'wizard.TITLE': 'Мастер', TITLE: 'Не то'}});
    const strings = {TITLE: 'Wizard'};

    localize('react-test', strings, 'wizard');

    expect(strings.TITLE).toBe('Мастер');
  });

  it('only reads its own dictionary', () => {
    withTranslations({'react-other': {TITLE: 'Пользователи'}});
    const strings = {TITLE: 'Users'};

    localize('react-test', strings);

    expect(strings.TITLE).toBe('Users');
  });

  it('fills the placeholders of a translated template', () => {
    withTranslations({'react-test': {EDIT: 'Изменить {0} ({1})'}});
    const strings = {EDIT: (name, type) => `Edit ${name} (${type})`};

    localize('react-test', strings);

    expect(strings.EDIT('maven-central', 'proxy')).toBe('Изменить maven-central (proxy)');
  });

  it('keeps a template English when an argument is an element', () => {
    withTranslations({'react-test': {EDIT: 'Изменить {0}'}});
    const strings = {EDIT: (name) => `Edit ${name}`};

    localize('react-test', strings);

    expect(strings.EDIT(<b>x</b>)).toBe('Edit [object Object]');
  });

  it('puts the elements of a fragment around the translated text', () => {
    withTranslations({'react-test': {HELP: 'Подробнее — в <0>документации</0>.<1/>'}});
    const strings = {
      HELP: <>See our <a href="https://example.com/docs">documentation</a> for more.<br/></>
    };

    localize('react-test', strings);
    render(<div data-testid="help">{strings.HELP}</div>);

    expect(screen.getByTestId('help')).toHaveTextContent('Подробнее — в документации.');
    expect(screen.getByRole('link', {name: 'документации'})).toHaveAttribute('href', 'https://example.com/docs');
    expect(screen.getByTestId('help').querySelector('br')).not.toBeNull();
  });

  it('numbers nested elements in document order', () => {
    withTranslations({'react-test': {TEXT: '<1>курсив</1> внутри <0>жирного <1>курсива</1></0>'}});
    const strings = {TEXT: <>Bold <b>and <i>italic</i></b></>};

    localize('react-test', strings);
    render(<div data-testid="text">{strings.TEXT}</div>);

    const text = screen.getByTestId('text');
    expect(text).toHaveTextContent('курсив внутри жирного курсива');
    expect(text.querySelector('b i')).toHaveTextContent('курсива');
  });

  it('translates a function returning an element and inserts its arguments', () => {
    withTranslations({'react-test': {DELETED: 'Репозиторий <0>{0}</0> удалён'}});
    const strings = {DELETED: (name) => <>Repository <strong>{name}</strong> deleted</>};

    localize('react-test', strings);
    render(<div data-testid="text">{strings.DELETED('npm-proxy')}</div>);

    expect(screen.getByTestId('text')).toHaveTextContent('Репозиторий npm-proxy удалён');
    expect(screen.getByTestId('text').querySelector('strong')).toHaveTextContent('npm-proxy');
  });

  it('keeps the English element when the translation does not fit it', () => {
    withTranslations({'react-test': {A: 'Сломано <0>без закрытия', B: 'Лишний <5/>', C: 'Чужой </0>'}});
    const element = <>Text <b>bold</b></>;
    const strings = {A: element, B: element, C: element};

    localize('react-test', strings);

    expect(strings.A).toBe(element);
    expect(strings.B).toBe(element);
    expect(strings.C).toBe(element);
  });

  it('ignores empty translations and values that are not texts', () => {
    withTranslations({'react-test': {TITLE: '', COUNT: '5'}});
    const strings = {TITLE: 'Users', COUNT: 3};

    localize('react-test', strings);

    expect(strings.TITLE).toBe('Users');
    expect(strings.COUNT).toBe(3);
  });

  it('survives cycles and frozen objects', () => {
    withTranslations({'react-test': {'A.TITLE': 'Заголовок', 'FROZEN.TITLE': 'Нельзя'}});
    const a = {TITLE: 'Title'};
    a.SELF = a;
    const frozen = Object.freeze({TITLE: 'Frozen'});

    localize('react-test', {A: a, FROZEN: frozen});

    expect(a.TITLE).toBe('Заголовок');
    expect(frozen.TITLE).toBe('Frozen');
  });

  it('replaces texts computed by getters with their translations', () => {
    withTranslations({
      'react-test': {
        STRING: 'Я подтверждаю',
        SUBLABEL: 'Введите «Я подтверждаю»',
        CONFIRM: 'Введите <0>Сбросить</0>'
      }
    });
    const strings = {
      STRING: 'I acknowledge',
      get SUBLABEL() {
        return `Type "${this.STRING}"`;
      },
      get CONFIRM() {
        return <>Type <strong>Reset</strong></>;
      }
    };

    localize('react-test', strings);
    render(<div data-testid="confirm">{strings.CONFIRM}</div>);

    expect(strings.SUBLABEL).toBe('Введите «Я подтверждаю»');
    expect(screen.getByTestId('confirm')).toHaveTextContent('Введите Сбросить');
    expect(screen.getByTestId('confirm').querySelector('strong')).toHaveTextContent('Сбросить');
  });

  it('keeps a text it cannot replace and translates the rest', () => {
    withTranslations({'react-test': {LOCKED: 'Заблокировано', TITLE: 'Заголовок'}});
    const strings = {TITLE: 'Title'};
    Object.defineProperty(strings, 'LOCKED', {value: 'Locked', enumerable: true, writable: false, configurable: false});

    expect(() => localize('react-test', strings)).not.toThrow();
    expect(strings.LOCKED).toBe('Locked');
    expect(strings.TITLE).toBe('Заголовок');
  });

  it('keeps a getter that throws and translates the rest', () => {
    withTranslations({'react-test': {BROKEN: 'Сломано', TITLE: 'Заголовок'}});
    const strings = {
      get BROKEN() {
        throw new Error('broken');
      },
      TITLE: 'Title'
    };

    expect(() => localize('react-test', strings)).not.toThrow();
    expect(strings.TITLE).toBe('Заголовок');
  });
});
