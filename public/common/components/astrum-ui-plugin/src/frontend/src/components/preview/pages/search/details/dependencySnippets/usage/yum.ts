import { usageStrings } from './strings';
import { encodePath, fileNameOf, isHosted, withSlash } from './urls';
import type { UsageContext, UsageGenerator, UsageSnippet } from './types';

/**
 * The directories of the repository that hold the repodata of a package: as deep as a hosted
 * repository generates it, else the directory above a `Packages` directory, else the package's own.
 */
function repodataBase(context: UsageContext): string[] {
  const segments = (context.path ?? '').replace(/^\/+/, '').split('/').filter(Boolean);
  const directories = segments.slice(0, -1);
  const depth = context.repodataDepth;
  if (depth != null && depth >= 0 && depth <= directories.length) {
    return directories.slice(0, depth);
  }
  const packages = directories.lastIndexOf('Packages');
  return packages >= 0 ? directories.slice(0, packages) : directories;
}

/**
 * Yum usage: the .repo file for /etc/yum.repos.d, dnf install and, for a hosted repository,
 * uploading a package.
 */
export const generate: UsageGenerator = (context) => {
  const { name, version, path, repositoryName } = context;
  const url = withSlash(context.repositoryUrl);
  const strings = usageStrings();
  const base = repodataBase(context);
  const baseUrl = base.length ? `${url}${encodePath(base.join('/'))}/` : url;
  const lines = [`[${repositoryName}]`, `name=${repositoryName}`, `baseurl=${baseUrl}`, 'enabled=1', 'gpgcheck=0'];
  if (context.repodataSigned) {
    lines.push('repo_gpgcheck=1', `gpgkey=${baseUrl}repodata/repomd.xml.key`);
  }

  const snippets: UsageSnippet[] = [
    {
      id: 'repo',
      label: `${repositoryName}.repo`,
      snippetText: lines.join('\n'),
      description: strings.HINTS.YUM_REPO(repositoryName),
    },
  ];
  if (name) {
    snippets.push({
      id: 'install',
      label: 'dnf',
      snippetText: `sudo dnf install ${name}${version ? `-${version}` : ''}`,
    });
  }
  if (isHosted(context.repositoryType)) {
    const file = path?.endsWith('.rpm') ? fileNameOf(path) : '<package>.rpm';
    const target = path?.endsWith('.rpm') ? encodePath(path) : `${base.length ? `${base.join('/')}/` : ''}<package>.rpm`;
    snippets.push({
      id: 'upload',
      label: strings.LABELS.UPLOAD,
      snippetText: `curl -u <user> --upload-file ./${file} ${url}${target}`,
      description: strings.HINTS.UPLOAD_WRITE_ACCESS,
    });
  }
  return snippets;
};
