import { usageStrings } from './strings';
import { hostnameOf, isHosted, isPlainHttp, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * PyPI usage: pip against the repository's simple index, pip.conf, requirements.txt and, for a
 * hosted repository, twine. A server on plain HTTP is only used by pip when the host is trusted.
 */
export const generate: UsageGenerator = (context) => {
  const { name, version } = context;
  if (!name) {
    return [];
  }
  const url = withSlash(context.repositoryUrl);
  const index = `${url}simple/`;
  const strings = usageStrings();
  const trusted = isPlainHttp(url) ? hostnameOf(url) : '';
  const requirement = version ? `${name}==${version}` : name;

  const snippets: UsageSnippet[] = [
    {
      id: 'pip',
      label: 'pip',
      snippetText: `pip install --index-url ${index}${trusted ? ` --trusted-host ${trusted}` : ''} ${requirement}`,
    },
    {
      id: 'pip-conf',
      label: 'pip.conf',
      snippetText: `[global]\nindex-url = ${index}${trusted ? `\ntrusted-host = ${trusted}` : ''}`,
      description: strings.HINTS.PIP_CONF,
    },
    {
      id: 'requirements',
      label: 'requirements.txt',
      snippetText: `--index-url ${index}${trusted ? `\n--trusted-host ${trusted}` : ''}\n${requirement}`,
    },
  ];
  if (isHosted(context.repositoryType)) {
    snippets.push({
      id: 'twine',
      label: 'twine',
      snippetText: `twine upload --repository-url ${url} dist/*`,
      description: strings.HINTS.TWINE,
    });
  }
  return snippets;
};
