import type { Snippet } from './snippets';

function secret(name: string): string {
  return `\${{ secrets.${name} }}`;
}

export function nx(repositoryUrl: string): Snippet[] {
  const url = repositoryUrl.replace(/\/+$/, '');
  return [
    {
      id: 'environment',
      label: 'Environment',
      code: [
        `export NX_SELF_HOSTED_REMOTE_CACHE_SERVER=${url}`,
        'export NX_SELF_HOSTED_REMOTE_CACHE_ACCESS_TOKEN=<access token>',
      ].join('\n'),
      hint: 'browse.usage.hint.nxEnvironment',
    },
    {
      id: 'ci',
      label: 'GitHub Actions',
      code: [
        'env:',
        `  NX_SELF_HOSTED_REMOTE_CACHE_SERVER: ${url}`,
        `  NX_SELF_HOSTED_REMOTE_CACHE_ACCESS_TOKEN: ${secret('ASTRUM_TOKEN')}`,
      ].join('\n'),
      hint: 'browse.usage.hint.nxCi',
    },
    { id: 'run', label: 'nx run-many', code: 'npx nx run-many -t build' },
  ];
}
