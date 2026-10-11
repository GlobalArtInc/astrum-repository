import { usageStrings } from './strings';
import { encodePath, fileNameOf, isHosted, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * Raw usage: download a file with curl or wget and, for a hosted repository, upload one to the same
 * path. Raw content is addressed by path only, so there is nothing to show without one.
 */
export const generate: UsageGenerator = (context) => {
  if (!context.path) {
    return [];
  }
  const fileUrl = `${withSlash(context.repositoryUrl)}${encodePath(context.path)}`;
  const file = fileNameOf(context.path);
  const strings = usageStrings();

  const snippets: UsageSnippet[] = [
    {
      id: 'curl',
      label: 'curl',
      snippetText: `curl -fLO ${fileUrl}`,
      description: strings.HINTS.RAW_SIGN_IN,
    },
    { id: 'wget', label: 'wget', snippetText: `wget ${fileUrl}`, description: strings.HINTS.RAW_SIGN_IN },
  ];
  if (isHosted(context.repositoryType)) {
    snippets.push({
      id: 'upload',
      label: strings.LABELS.UPLOAD,
      snippetText: `curl -u <user> --upload-file ./${file} ${fileUrl}`,
      description: strings.HINTS.UPLOAD_WRITE_ACCESS,
    });
  }
  return snippets;
};
