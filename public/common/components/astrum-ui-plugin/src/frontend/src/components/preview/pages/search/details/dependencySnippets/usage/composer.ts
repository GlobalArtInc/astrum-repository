import { usageStrings } from './strings';
import { isHosted, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * Composer usage: register the repository, require the package, composer.json and, for a hosted
 * repository, uploading a package archive.
 */
export const generate: UsageGenerator = (context) => {
  const { group, name, version, repositoryName } = context;
  const url = withSlash(context.repositoryUrl);
  const strings = usageStrings();
  const pkg = name ? (group ? `${group}/${name}` : name) : '';

  const snippets: UsageSnippet[] = [
    {
      id: 'config',
      label: 'composer config',
      snippetText: `composer config repositories.${repositoryName} composer ${url}`,
      description: strings.HINTS.COMPOSER_CONFIG,
    },
  ];
  if (pkg) {
    snippets.push({ id: 'require', label: 'composer require', snippetText: `composer require ${pkg}${version ? `:${version}` : ''}` });
  }
  snippets.push({
    id: 'json',
    label: 'composer.json',
    snippetText:
      '"repositories": [\n' +
      `  { "type": "composer", "url": "${url}" }\n` +
      ']',
  });
  if (isHosted(context.repositoryType) && pkg && version) {
    snippets.push({
      id: 'upload',
      label: strings.LABELS.UPLOAD,
      snippetText: `curl -u <user> --upload-file package.zip ${url}packages/upload/${pkg}/${version}`,
      description: strings.HINTS.UPLOAD_WRITE_ACCESS,
    });
  }
  return snippets;
};
