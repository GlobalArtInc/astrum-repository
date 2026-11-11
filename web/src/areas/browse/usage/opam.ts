import type { Snippet, UsageContext } from './snippets';

export function opam(context: UsageContext, repositoryUrl: string): Snippet[] {
  const root = repositoryUrl.replace(/\/+$/, '');
  const name = context.repositoryName;
  const hosted = context.repositoryType === 'hosted';
  const component = context.component?.name || '<package>';
  const version = context.component?.version || '<version>';
  const directory = `${root}/packages/${component}/${component}.${version}`;
  const snippets: Snippet[] = [
    {
      id: 'add',
      label: 'opam repo add',
      code: `opam repo add ${name} ${root} --all-switches --set-default`,
      hint: 'browse.usage.hint.opamAdd',
    },
  ];
  if (hosted) {
    snippets.push({
      id: 'upload',
      label: 'curl -T',
      code: [
        `curl -u <user>:<access token> -T opam ${directory}/opam`,
        `curl -u <user>:<access token> -T <archive> ${root}/cache/sha256/<first two characters>/<sha256 of the archive>`,
      ].join('\n'),
      hint: 'browse.usage.hint.opamUpload',
    });
  } else {
    snippets.push({
      id: 'remove',
      label: 'opam repo remove',
      code: 'opam repo remove default --all',
      hint: 'browse.usage.hint.opamRemove',
    });
  }
  return snippets;
}
