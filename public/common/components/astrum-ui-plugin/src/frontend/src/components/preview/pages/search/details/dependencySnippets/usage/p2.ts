import { usageStrings } from './strings';
import { withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

function xml(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

/**
 * p2 usage: the repository as an Eclipse update site and as a Tycho repository.
 */
export const generate: UsageGenerator = (context) => {
  const url = withSlash(context.repositoryUrl);
  const strings = usageStrings();

  const snippets: UsageSnippet[] = [
    { id: 'site', label: strings.LABELS.UPDATE_SITE, snippetText: url, description: strings.HINTS.P2_SITE },
    {
      id: 'tycho',
      label: 'Tycho',
      snippetText:
        '<repository>\n' +
        `  <id>${xml(context.repositoryName)}</id>\n` +
        '  <layout>p2</layout>\n' +
        `  <url>${xml(url)}</url>\n` +
        '</repository>',
      description: strings.HINTS.TYCHO,
    },
  ];
  return snippets;
};
