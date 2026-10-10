# Interface language

The web interface is available in English and Russian.

The server picks the language for each browser:

1. the language chosen in the header's language menu (kept in the `astrum-language` cookie for a year);
2. otherwise the first language in the browser's preferred languages (`Accept-Language`) that the
   server has a translation for;
3. otherwise English.

To switch, open the language menu (the icon next to Help in the header) and pick a language; the page
reloads in it. Texts that are not translated yet are shown in English.

The language only changes what the interface shows. Repository content, logs, REST API responses and
task names stay in English.
