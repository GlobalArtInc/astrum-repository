import { usageStrings } from './strings';
import { isHosted, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * R usage: install.packages from the repository, a specific version with remotes, .Rprofile and,
 * for a hosted repository, uploading a source package.
 */
export const generate: UsageGenerator = (context) => {
  const { name, version, repositoryName } = context;
  const url = withSlash(context.repositoryUrl);
  const strings = usageStrings();

  const snippets: UsageSnippet[] = [];
  if (name) {
    snippets.push({ id: 'install', label: 'install.packages', snippetText: `install.packages("${name}", repos = "${url}")` });
    if (version) {
      snippets.push({
        id: 'remotes',
        label: 'remotes',
        snippetText: `remotes::install_version("${name}", version = "${version}", repos = "${url}")`,
      });
    }
  }
  snippets.push({
    id: 'rprofile',
    label: '.Rprofile',
    snippetText: `options(repos = c(${repositoryName.replace(/[^A-Za-z0-9_.]/g, '_')} = "${url}"))`,
    description: strings.HINTS.RPROFILE,
  });
  if (isHosted(context.repositoryType) && name && version) {
    const file = `${name}_${version}.tar.gz`;
    snippets.push({
      id: 'upload',
      label: strings.LABELS.UPLOAD,
      snippetText: `curl -u <user> --upload-file ${file} ${url}src/contrib/${file}`,
      description: strings.HINTS.UPLOAD_WRITE_ACCESS,
    });
  }
  return snippets;
};
