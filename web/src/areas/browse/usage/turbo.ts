import type { Snippet } from './snippets';

export const TURBO_TEAM = '<team>';

function secret(name: string): string {
  return `\${{ secrets.${name} }}`;
}

export function turbo(repositoryUrl: string): Snippet[] {
  const url = repositoryUrl.replace(/\/+$/, '');
  return [
    {
      id: 'environment',
      label: 'Environment',
      code: [
        `export TURBO_API=${url}`,
        'export TURBO_TOKEN=<access token>',
        `export TURBO_TEAM=${TURBO_TEAM}`,
      ].join('\n'),
      hint: 'browse.usage.hint.turboEnvironment',
    },
    {
      id: 'config',
      label: 'turbo.json',
      code: JSON.stringify(
        {
          $schema: 'https://turborepo.dev/schema.json',
          remoteCache: { apiUrl: url, teamSlug: TURBO_TEAM, signature: true },
        },
        null,
        2,
      ),
      hint: 'browse.usage.hint.turboConfig',
    },
    {
      id: 'ci',
      label: 'GitHub Actions',
      code: [
        'env:',
        `  TURBO_API: ${url}`,
        `  TURBO_TOKEN: ${secret('ASTRUM_TOKEN')}`,
        `  TURBO_TEAM: ${TURBO_TEAM}`,
        `  TURBO_REMOTE_CACHE_SIGNATURE_KEY: ${secret('TURBO_SIGNATURE_KEY')}`,
      ].join('\n'),
      hint: 'browse.usage.hint.turboCi',
    },
    { id: 'run', label: 'turbo run', code: 'turbo run build --summarize' },
  ];
}
