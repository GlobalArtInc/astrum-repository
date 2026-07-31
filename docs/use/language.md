# Interface language

The web interface is available in English and Russian.

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
