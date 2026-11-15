# Interface translations

The texts of the web UI live in `web/src/**/strings/en.json` and `ru.json`, one pair per area (see
[`web/README.md`](../../web/README.md)). Their translations are managed in [Ownlate](https://ownlate.com)
and pulled into the JSON dictionaries of the web UI while the image is built (the `web` stage of the
`Dockerfile`). The translations are not in git: only the English source is. A running server does not talk
to Ownlate.

## Dictionaries

[`dictionaries.json`](dictionaries.json) makes every `strings` directory under `web/src` that holds an
`en.json` a dictionary with the Ownlate prefix `web`. Each one holds:

- `en.json`: the English source, written by hand (it is the source itself);
- `<lang>.json`: a translation, written by `pull` (never edited by hand, never committed; git ignores them).

In Ownlate one project holds all dictionaries:

- The Ownlate key is `web.` and the key, e.g. `web.browse.list.title`.
- A plural entry is one Ownlate key per form, `<key>#<form>`, e.g. `web.browse.list.count#few`. The source
  offers every form a translation needs (`one`, `few`, `many` and `other` for Russian); a form English
  lacks carries its `other` text.
- `pull` writes `ru.json` in the key order and with the blank lines of its `en.json`, plural forms in CLDR
  order, and takes a plural entry only with all the forms Russian needs.
- `check` compares the `{name}` placeholders of each entry (over all its plural forms) with `en.json` and
  the plural forms with those of the language. `pnpm i18n:check` in `web/` checks the rest: every key
  translated, every key used in the code present.

## Changing texts

Developers write the Russian text next to the English one. Upload both after a change: the source texts
first, then the translations as approved (`catalogue en` and `catalogue ru` print them under their Ownlate
keys); then create a release and `pull` to check them locally.

`import <lang> <file>` rewrites each dictionary with only the keys in the file, so build the file from
`catalogue <lang>` plus the new keys.

```bash
node tools/i18n/i18n.mjs catalogue en
node tools/i18n/i18n.mjs catalogue ru
```

## Pulling translations

```bash
node tools/i18n/i18n.mjs pull
node tools/i18n/i18n.mjs check
```

`pull` needs no token. It reads the published release of the Ownlate project: the manifest of the release
lists the languages, and each language is fetched as its own bundle. A text reaches `pull` only through a
release, so after importing or approving texts in Ownlate, create a release (`create_release`); without it
`pull` and CI do not see the texts. `pull` rewrites every `<lang>.json` from the release; keys the code no
longer has are dropped.

`pull` used to ask Ownlate for every language in one answer (`segments/translations-map`). The gateway
refuses answers above 4 MiB, and the project has passed that size, so the one answer is no longer served.

Without network access to Ownlate, export a language from the web interface as JSON
and run `node tools/i18n/i18n.mjs import <lang> <file>` instead. Like `pull`, `import` rewrites every
dictionary from what it is given, so give it a whole export of the language, never a part.

`node tools/i18n/i18n.mjs status` prints how much of each dictionary is translated.

## Adding a language

Add the language to the Ownlate project, translate, create a release, and pull. The web UI lists its languages in
`LANGUAGES` of `web/src/i18n/translate.ts` and `web/scripts/i18n-check.mjs`. When the new language has
plural forms the others lack, upload `catalogue en` again once its `<lang>.json` files exist, so that
Ownlate offers those forms.

## CI and the image

The `i18n` workflow runs `check` on every change. When the repository has the `ASTRUM_OWNLATE_TOKEN`
secret and the `ASTRUM_OWNLATE_PROJECT` variable, it also pulls and checks the translations. `pull` itself
ignores the token.

The `Dockerfile` pulls them in the `web` stage. `pull` ignores the token, but the stage still reads it from
a build secret, so the build fails without it:

```bash
docker build --secret id=ownlate_token,env=ASTRUM_OWNLATE_TOKEN -t astrum-server:local .
```
