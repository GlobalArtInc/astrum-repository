import { usageStrings } from './strings';
import { fileNameOf, isHosted, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * APT usage: the sources.list line, apt-get install and, for a hosted repository, uploading a
 * package. The distribution comes from the repository settings, or from a `dists/` path.
 */
export const generate: UsageGenerator = (context) => {
  const { name, version, path } = context;
  const url = withSlash(context.repositoryUrl);
  const strings = usageStrings();
  const fromPath = path ? /(?:^|\/)dists\/([^/]+)\//.exec(path)?.[1] : undefined;
  const distribution = context.distribution || fromPath || '<distribution>';

  const snippets: UsageSnippet[] = [
    {
      id: 'sources',
      label: 'sources.list',
      snippetText: `deb ${url} ${distribution} main`,
      description: strings.HINTS.APT_SOURCES(context.repositoryName),
    },
  ];
  if (name) {
    snippets.push({
      id: 'install',
      label: 'apt-get',
      snippetText: `sudo apt-get update\nsudo apt-get install ${name}${version ? `=${version}` : ''}`,
    });
  }
  if (isHosted(context.repositoryType)) {
    const file = path?.endsWith('.deb') ? fileNameOf(path) : '<package>.deb';
    snippets.push({
      id: 'upload',
      label: strings.LABELS.UPLOAD,
      snippetText: `curl -u <user> -H "Content-Type: multipart/form-data" --data-binary "@${file}" ${url}`,
      description: strings.HINTS.UPLOAD_WRITE_ACCESS,
    });
  }
  return snippets;
};
