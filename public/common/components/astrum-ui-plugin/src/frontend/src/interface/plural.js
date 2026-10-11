/**
 * Returns the language the page is shown in: the language of its translations, else English.
 */
export function pageLanguage() {
  const i18n = typeof window === 'undefined' ? undefined : window.AS_I18N;
  return (i18n && i18n.language) || 'en';
}

/**
 * Picks the plural form of a text for a count, by the plural rules of the page's language.
 *
 * `forms` holds a form per plural category of the languages the texts are translated to, `ONE`, `FEW`, `MANY` and
 * `OTHER`, each a text or a function of the count, so that every language finds its forms among the keys of the
 * English texts. A missing category falls back to `OTHER`. A function gets `shown`, the count as it is to be written,
 * which is the count itself unless given.
 */
export default function plural(forms, count, shown = count) {
  let category = 'other';
  try {
    category = new Intl.PluralRules(pageLanguage()).select(count);
  }
  catch (e) {
    category = count === 1 ? 'one' : 'other';
  }
  const form = forms[category.toUpperCase()] ?? forms.OTHER;
  return typeof form === 'function' ? form(shown) : form;
}
