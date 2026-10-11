import { usageStrings } from './strings';
import { withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * CocoaPods usage: the Podfile source and pod line, and registering the repository as a CDN source.
 */
export const generate: UsageGenerator = (context) => {
  const { name, version, repositoryName } = context;
  const url = withSlash(context.repositoryUrl);
  const strings = usageStrings();
  const pod = name ? `\npod '${name}'${version ? `, '${version}'` : ''}` : '';

  const snippets: UsageSnippet[] = [
    { id: 'podfile', label: 'Podfile', snippetText: `source '${url}'\n${pod}`.replace(/\n$/, '') },
    {
      id: 'repo',
      label: 'pod repo',
      snippetText: `pod repo add-cdn ${repositoryName} ${url}`,
      description: strings.HINTS.POD_REPO,
    },
  ];
  return snippets;
};
