import { usageStrings } from './strings';
import { isHosted, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * npm usage: install with npm, Yarn or pnpm, the .npmrc registry line and, for a hosted repository,
 * publishing. A scoped package gets a scoped registry line, so other packages keep their registry.
 */
export const generate: UsageGenerator = (context) => {
  const { group, name, version } = context;
  if (!name) {
    return [];
  }
  const url = withSlash(context.repositoryUrl);
  const strings = usageStrings();
  const scope = group ? `@${group.replace(/^@/, '')}` : '';
  const pkg = scope ? `${scope}/${name}` : name;
  const spec = version ? `${pkg}@${version}` : pkg;

  const snippets: UsageSnippet[] = [
    { id: 'npm', label: 'npm', snippetText: `npm install ${spec} --registry=${url}` },
    { id: 'yarn', label: 'Yarn', snippetText: `yarn add ${spec} --registry ${url}` },
    { id: 'pnpm', label: 'pnpm', snippetText: `pnpm add ${spec} --registry=${url}` },
    {
      id: 'npmrc',
      label: '.npmrc',
      snippetText: scope ? `${scope}:registry=${url}` : `registry=${url}`,
      description: strings.HINTS.NPMRC,
    },
  ];
  if (isHosted(context.repositoryType)) {
    snippets.push({
      id: 'publish',
      label: 'npm publish',
      snippetText: `npm login --registry=${url}\nnpm publish --registry=${url}`,
      description: strings.HINTS.NPM_PUBLISH,
    });
  }
  return snippets;
};
