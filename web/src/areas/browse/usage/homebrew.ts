import type { Snippet, UsageContext } from './snippets';

export function homebrew(context: UsageContext, repositoryUrl: string): Snippet[] {
  const root = repositoryUrl.replace(/\/+$/, '');
  const formula = context.component?.name || '<formula>';
  return [
    {
      id: 'environment',
      label: 'Environment',
      code: [
        `export HOMEBREW_API_DOMAIN=${root}/api`,
        `export HOMEBREW_ARTIFACT_DOMAIN=${root}`,
        'export HOMEBREW_ARTIFACT_DOMAIN_NO_FALLBACK=1',
      ].join('\n'),
      hint: 'browse.usage.hint.homebrewEnvironment',
    },
    {
      id: 'install',
      label: 'brew install',
      code: `brew install ${formula}`,
      hint: 'browse.usage.hint.homebrewInstall',
    },
    {
      id: 'private',
      label: 'Private repository',
      code: 'export HOMEBREW_DOCKER_REGISTRY_TOKEN=<access token>',
      hint: 'browse.usage.hint.homebrewPrivate',
    },
  ];
}
