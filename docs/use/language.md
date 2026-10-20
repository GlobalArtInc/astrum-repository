# Interface language

The web interface is available in English, Russian, German, Spanish, French, Italian, Japanese, Polish,
Portuguese (Brazil), Serbian (Latin script), Turkish and Simplified Chinese.

The interface picks the language for each browser:

1. the language chosen in the language menu of the header (kept in the `astrum-language` cookie);
2. otherwise the first language in the browser's preferred languages (`Accept-Language`) that the
   interface has a translation for;
3. otherwise English.

To switch, open the language menu in the header and pick a language; the interface changes at once,
without reloading the page. Texts that are not translated yet are shown in English.

The language only changes what the interface shows. Repository content, logs, REST API responses and
some texts that come from the server, such as task names and the descriptions of built-in privileges,
stay in English.

## Help with translations

The texts of the interface are translated in [Ownlate](https://platform.ownlate.com/workspaces/c08d5107-8ec7-4c80-9f46-e36fa7eeab3a/projects/f1f93c0e-4902-434d-a551-e94ba7e1e304/progress), where the progress of every language is
shown. The language menu in the header links there too, under **Help translate**. A corrected or new
translation reaches the interface with the next build of the server, which takes the approved
translations from Ownlate.
