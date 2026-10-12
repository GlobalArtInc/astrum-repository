# Interface translations

The English texts of the interface live in the code. Their translations are managed in
[Ownlate](https://ownlate.com) and committed to this repository as JSON dictionaries, which the build
packs into the modules. A running server does not talk to Ownlate.

## Dictionaries

[`dictionaries.json`](dictionaries.json) lists the dictionaries. Each one of the Java UIs is a directory
`<module>/src/main/resources/astrum-i18n/<name>/` holding:

- `en.json`: the English source, generated from the code by `extract` (never edited by hand);
- `<lang>.json`: a translation, written by `pull` (never edited by hand either).

In Ownlate every key carries the dictionary's `prefix`, e.g. `extjs.coreui.Users_Create_Title`, so
one project holds all dictionaries.

| Dictionary | Source in the code |
|---|---|
| `extjs-rapture` | `astrum-rapture/.../AS/app/PluginStrings.js` |
| `extjs-coreui` | `astrum-coreui-plugin/.../AS/coreui/app/PluginStrings.js` |
| `extjs-onboarding` | `astrum-onboarding-plugin/.../AS/onboarding/app/PluginStrings.js` |
| `react-ui` | `astrum-ui-plugin/src/frontend/src/constants/UIStrings.js`, the onboarding wizard's `UIStrings.ts` and Astrum One's `components/preview/constants/PreviewStrings.js` |
| `react-coreui` | `astrum-coreui-plugin/src/frontend/src/constants/UIStrings.jsx` and the Astrum One route titles in `routerConfig/routes/previewRouteTitles.js` |
| `web` | `web/src/**/strings/en.json` of the web UI (below) |

ExtJS keys are the keys of `PluginStrings.js`; bundle keys are written `<class>/<key>`. Values that only
reference another key (`@Other_Key`) or contain no letters are not translated.

## Web UI texts

The web UI of the Rust server keeps its texts in `web/src/**/strings/en.json` and `ru.json`, one pair per
area (see [`web/README.md`](../../web/README.md)). The `web` entry of `dictionaries.json` makes every
`strings` directory under `web/src` a dictionary with the prefix `web`. Its `en.json` is written by hand
and is the source itself, so `extract` leaves it alone.

- The Ownlate key is `web.` and the key, e.g. `web.browse.list.title`.
- A plural entry is one Ownlate key per form, `<key>#<form>`, e.g. `web.browse.list.count#few`. The source
  offers every form a translation needs (`one`, `few`, `many` and `other` for Russian); a form English
  lacks carries its `other` text.
- `pull` writes `ru.json` in the key order and with the blank lines of its `en.json`, plural forms in CLDR
  order, and takes a plural entry only with all the forms Russian needs.
- `check` compares the `{name}` placeholders of each entry (over all its plural forms) with `en.json` and
  the plural forms with those of the language. `pnpm i18n:check` in `web/` checks the rest: every key
  translated, every key used in the code present.

Developers write the Russian text next to the English one. Upload both after a change: the source texts
first, then the translations as approved (`catalogue en` and `catalogue ru` print them under their Ownlate
keys); then `pull` and commit what it changes.

## React texts

A React strings object is passed to `localize('<dictionary>', strings)` (exported by
`@globalart/astrum-ui-plugin`), which replaces its texts in place with the translations of the page's
language. A key is the dotted path of the text in the object, e.g. `USERS.LIST.CREATE_BUTTON`.

- A string is translated as is.
- A function returning a text, `(name) => \`Edit ${name}\``, is translated as `Edit {0}`; the
  translation fills `{0}`, `{1}`... with the arguments.
- A JSX element, or a function returning one, is translated as its text with the descendant
  elements numbered in document order: `See <0>the documentation</0>.<1/>`. The translation keeps
  the elements (links, bold text) and may move them.

A module passed with a scope, `localize('react-ui', strings, 'PREVIEW.TASKS')`, has its keys under the scope,
e.g. `PREVIEW.TASKS.LIST.CREATE_TASK`. The strings modules of Astrum One localize themselves this way, each under
`PREVIEW.<AREA>`, so that a component importing one directly always reads the translations; `PreviewStrings.js` lists
them. A text that depends on a count is an object of plural forms, `ONE`, `FEW`, `MANY` and `OTHER` (texts or
templates of the count), from which `plural()` (`interface/plural.js`) picks the form the page's language needs.

`extract` loads the modules in Node to read these texts. It leaves English, and lists, the functions
whose text depends on their arguments in other ways (plurals, lists, conditions) and values that look
like identifiers (`snake_case`, `camelCase`, property names, values under `id`, `value`, `type`...).
Texts written straight into components are not translated: put them into a strings file first.

## How a page gets its language

The server serves the page in the language of the `astrum-language` cookie, else the first language the
browser accepts that has a dictionary, else English. For a language other than English the page loads
`static/rapture/i18n-<lang>.js` right after the framework, which sets `window.AS_I18N` to the
language's dictionaries and, when the module ships one, applies `astrum-i18n/ext-locale-<lang>.js`
(the framework's own texts: dates, paging, field validation, dialog buttons). `AS.I18n` then uses the
translations instead of the English texts. The language menu in the header and on the sign-in page sets
the cookie and reloads the page.

React texts are translated the same way: the strings objects read `window.AS_I18N` when their modules
load (see React texts above). A missing translation falls back to English, so a partly translated language still
works.

## Changing English texts

1. Change the text in the code and run

   ```bash
   node tools/i18n/i18n.mjs extract
   ```

2. Commit the code together with the regenerated `en.json`. CI runs `check`, which fails when an
   `en.json` is out of date.
3. Upload the new source texts to Ownlate (the CLI or the web interface), translate them there, then
   pull the translations (below).

Reading the React texts needs `typescript` and `react`: they come with `yarn install`, or install
them anywhere and point `ASTRUM_I18N_MODULES` at that directory.

`node tools/i18n/i18n.mjs catalogue en` prints every source text under its Ownlate key, the shape
Ownlate imports.

## Pulling translations

With an Ownlate API key in `ASTRUM_OWNLATE_TOKEN` and the project id in `ASTRUM_OWNLATE_PROJECT`:

```bash
node tools/i18n/i18n.mjs pull
node tools/i18n/i18n.mjs check
```

`pull` rewrites every `<lang>.json` from the project's approved translations; keys the code no longer
has are dropped. Without network access to Ownlate, export a language from the web interface as JSON
and run `node tools/i18n/i18n.mjs import <lang> <file>` instead. Like `pull`, `import` rewrites every
dictionary from what it is given, so give it a whole export of the language, never a part.

`check` also fails when a translation has a key the source lacks, or different `{0}` placeholders,
HTML tags or numbered `<0>` elements than the English text.

`node tools/i18n/i18n.mjs status` prints how much of each dictionary is translated.

## Adding a language

Add the language to the Ownlate project, translate, and pull. A language appears in the header menu as
soon as some module ships a dictionary for it. For the framework's own texts add
`astrum-rapture/src/main/resources/astrum-i18n/ext-locale-<lang>.js` (see `ext-locale-ru.js`). The web UI
also lists its languages in `LANGUAGES` of `web/src/i18n/translate.ts` and `web/scripts/i18n-check.mjs`;
when the new language has plural forms the others lack, upload `catalogue en` again once its `<lang>.json`
files exist, so that Ownlate offers those forms.

## CI

The `i18n` workflow runs `check` on every change. When the repository has the `ASTRUM_OWNLATE_TOKEN`
secret and the `ASTRUM_OWNLATE_PROJECT` variable, it also pulls and fails if the committed translations,
those of the web UI included, differ from Ownlate.
