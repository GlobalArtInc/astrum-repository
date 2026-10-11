import { usageStrings } from './strings';
import { encodePath, isHosted, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * Channel of a Conda asset: its path minus the platform directory and the file name
 * (`main/linux-64/numpy-1.26.tar.bz2` is in channel `main`).
 */
function channelOf(path: string | null | undefined): string {
  const segments = (path ?? '').replace(/^\/+/, '').split('/').filter(Boolean);
  return segments.length >= 3 ? segments.slice(0, -2).join('/') : '';
}

/**
 * Conda usage: install from the repository's channel, .condarc and, for a hosted repository,
 * uploading a package file.
 */
export const generate: UsageGenerator = (context) => {
  const { name, version, path } = context;
  const url = withSlash(context.repositoryUrl);
  const channel = channelOf(path);
  const channelUrl = channel ? `${url}${channel}` : url.replace(/\/$/, '');
  const strings = usageStrings();

  const snippets: UsageSnippet[] = [];
  if (name) {
    snippets.push({
      id: 'install',
      label: 'conda install',
      snippetText: `conda install --override-channels -c ${channelUrl} ${name}${version ? `=${version}` : ''}`,
    });
  }
  snippets.push({
    id: 'condarc',
    label: '.condarc',
    snippetText: `channels:\n  - ${channelUrl}`,
    description: strings.HINTS.CONDARC,
  });
  if (isHosted(context.repositoryType) && path) {
    snippets.push({
      id: 'upload',
      label: strings.LABELS.UPLOAD,
      snippetText: `curl -u <user> --upload-file ./${path.substring(path.lastIndexOf('/') + 1)} ${url}${encodePath(path)}`,
      description: strings.HINTS.UPLOAD_WRITE_ACCESS,
    });
  }
  return snippets;
};
