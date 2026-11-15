import type { Snippet } from './snippets';

export function sccache(repositoryUrl: string): Snippet[] {
  const url = repositoryUrl.replace(/\/+$/, '');
  return [
    {
      id: 'environment',
      label: 'Environment',
      code: [`SCCACHE_WEBDAV_ENDPOINT=${url}`, 'SCCACHE_WEBDAV_TOKEN=<access token>'].join('\n'),
      hint: 'browse.usage.hint.sccacheEnvironment',
    },
    { id: 'rust', label: 'Rust', code: 'RUSTC_WRAPPER=sccache cargo build' },
    { id: 'cc', label: 'C and C++', code: 'sccache gcc -c a.c -o a.o' },
    {
      id: 'developers',
      label: 'Developers',
      code: 'SCCACHE_WEBDAV_RW_MODE=READ_ONLY',
      hint: 'browse.usage.hint.sccacheDevelopers',
    },
  ];
}
