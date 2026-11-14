import type { Snippet } from './snippets';

export function bazelcache(repositoryUrl: string): Snippet[] {
  const url = repositoryUrl.replace(/\/+$/, '');
  const header = 'build --remote_header=Authorization=Bearer <access token>';
  return [
    {
      id: 'bazelrc',
      label: '.bazelrc',
      code: [`build --remote_cache=${url}`, header].join('\n'),
      hint: 'browse.usage.hint.bazelcacheRc',
    },
    {
      id: 'developers',
      label: 'Developers',
      code: 'build --remote_upload_local_results=false',
      hint: 'browse.usage.hint.bazelcacheDevelopers',
    },
    {
      id: 'ci',
      label: 'CI',
      code: 'build --remote_upload_local_results=true',
      hint: 'browse.usage.hint.bazelcacheCi',
    },
  ];
}
