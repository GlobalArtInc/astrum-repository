import { usageStrings } from './strings';
import { encodePath, isHosted, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * Alpine usage: the line for /etc/apk/repositories and apk add. The repository line is the
 * directory above the architecture directory of the package (`v3.20/main/x86_64/curl.apk`).
 */
export const generate: UsageGenerator = (context) => {
  const { name, version, path } = context;
  const url = withSlash(context.repositoryUrl);
  const strings = usageStrings();
  const segments = (path ?? '').replace(/^\/+/, '').split('/').filter(Boolean);
  const branch = segments.length >= 3 ? segments.slice(0, -2).join('/') : '';
  const line = branch ? `${url}${branch}` : url.replace(/\/$/, '');

  const snippets: UsageSnippet[] = [
    {
      id: 'repositories',
      label: '/etc/apk/repositories',
      snippetText: `echo "${line}" >> /etc/apk/repositories\napk update`,
      description: strings.HINTS.APK_REPOSITORIES,
    },
  ];
  if (name) {
    snippets.push({ id: 'add', label: 'apk add', snippetText: `apk add ${name}${version ? `=${version}` : ''}` });
  }
  if (isHosted(context.repositoryType) && path && path.endsWith('.apk')) {
    snippets.push({
      id: 'upload',
      label: strings.LABELS.UPLOAD,
      snippetText: `curl -u <user> --upload-file ./${segments[segments.length - 1]} ${url}${encodePath(path)}`,
      description: strings.HINTS.UPLOAD_WRITE_ACCESS,
    });
  }
  return snippets;
};
