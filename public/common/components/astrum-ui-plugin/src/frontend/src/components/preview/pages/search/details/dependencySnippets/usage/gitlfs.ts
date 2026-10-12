import { usageStrings } from './strings';
import { fileNameOf, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

const OID = /^[0-9a-f]{64}$/;

/**
 * Git LFS usage: point a git repository at this repository's LFS API and, for a stored object,
 * download it by its id the way git-lfs does.
 */
export const generate: UsageGenerator = (context) => {
  const api = `${withSlash(context.repositoryUrl)}info/lfs`;
  const strings = usageStrings();
  const oid = [context.name, context.path ? fileNameOf(context.path) : null].find(
    (candidate): candidate is string => !!candidate && OID.test(candidate),
  );

  const snippets: UsageSnippet[] = [
    {
      id: 'lfsconfig',
      label: '.lfsconfig',
      snippetText: `git config -f .lfsconfig lfs.url ${api}\ngit add .lfsconfig`,
      description: strings.HINTS.LFS_CONFIG,
    },
    {
      id: 'git-config',
      label: 'git config',
      snippetText: `git config lfs.url ${api}`,
      description: strings.HINTS.LFS_LOCAL_CONFIG,
    },
  ];
  if (oid) {
    snippets.push({
      id: 'curl',
      label: 'curl',
      snippetText: `curl -u <user> -fL -o ${oid} ${api}/objects/${oid}`,
      description: strings.HINTS.LFS_OBJECT,
    });
  }
  return snippets;
};
