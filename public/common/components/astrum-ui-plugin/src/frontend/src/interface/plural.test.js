import plural, {pageLanguage} from './plural';

const FORMS = {
  ONE: (count) => `${count} one`,
  FEW: (count) => `${count} few`,
  MANY: (count) => `${count} many`,
  OTHER: (count) => `${count} other`,
};

describe('plural', () => {
  afterEach(() => {
    delete window.AS_I18N;
  });

  it('uses the English rules when the page has no translations', () => {
    expect(pageLanguage()).toBe('en');
    expect(plural(FORMS, 1)).toBe('1 one');
    expect(plural(FORMS, 2)).toBe('2 other');
    expect(plural(FORMS, 5)).toBe('5 other');
  });

  it('uses the rules of the language of the page', () => {
    window.AS_I18N = {language: 'ru', dictionaries: {}};

    expect(pageLanguage()).toBe('ru');
    expect(plural(FORMS, 1)).toBe('1 one');
    expect(plural(FORMS, 21)).toBe('21 one');
    expect(plural(FORMS, 3)).toBe('3 few');
    expect(plural(FORMS, 5)).toBe('5 many');
    expect(plural(FORMS, 11)).toBe('11 many');
    expect(plural(FORMS, 1.5)).toBe('1.5 other');
  });

  it('takes texts as well as functions and falls back to the other form', () => {
    window.AS_I18N = {language: 'ru', dictionaries: {}};

    expect(plural({ONE: 'item', OTHER: 'items'}, 1)).toBe('item');
    expect(plural({ONE: 'item', OTHER: 'items'}, 5)).toBe('items');
  });

  it('writes the count as given', () => {
    expect(plural(FORMS, 1234, '1,234')).toBe('1,234 other');
  });
});
