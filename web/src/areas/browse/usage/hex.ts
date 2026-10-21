import type { Snippet, UsageContext } from './snippets';

const RELEASE = /^(\d+)\.(\d+)\.\d+$/;

export function hexRequirement(version: string): string {
  const release = RELEASE.exec(version);
  if (release) {
    return `~> ${release[1]}.${release[2]}`;
  }
  return version ? `== ${version}` : '~> 1.0';
}

export function mixDependency(name: string, version: string, repository?: string): string {
  const repo = repository ? `, repo: "${repository}"` : '';
  return `{:${name}, "${hexRequirement(version)}"${repo}}`;
}

export function rebarDependency(name: string, version: string): string {
  return version ? `{${name}, "${version}"}` : name;
}

function rebarRepository(repository: string, url: string, hosted: boolean): string {
  const lines = [
    `{hex, [{repos, [#{name => <<"${repository}">>,`,
    `                  repo_url => <<"${url}">>,`,
    `                  repo_public_key => <<"<contents of ${repository}.pem>">>${hosted ? ',' : '}]}]}.'}`,
  ];
  if (hosted) {
    lines.push(`                  api_url => <<"${url}/api">>}]}]}.`);
  }
  return lines.join('\n');
}

export function hex(context: UsageContext, address: string): Snippet[] {
  const url = address.replace(/\/+$/, '');
  const repository = context.repositoryName;
  const name = context.component?.name || '<package>';
  const version = context.component?.version ?? '';
  if (context.repositoryType === 'proxy') {
    return [
      {
        id: 'mirror',
        label: 'HEX_MIRROR',
        code: `export HEX_MIRROR=${url}\nmix deps.get`,
        hint: 'browse.usage.hint.hexMirror',
      },
      {
        id: 'rebar3',
        label: 'HEX_CDN',
        code: `export HEX_CDN=${url}\nrebar3 get-deps`,
        hint: 'browse.usage.hint.hexCdn',
      },
      { id: 'dependency', label: 'mix.exs', code: mixDependency(name, version) },
    ];
  }
  const hosted = context.repositoryType === 'hosted';
  const snippets: Snippet[] = [
    {
      id: 'repo',
      label: 'mix hex.repo add',
      code: [
        `curl -fsS -o ${repository}.pem ${url}/public_key`,
        `mix hex.repo add ${repository} ${url} --public-key ${repository}.pem`,
      ].join('\n'),
      hint: 'browse.usage.hint.hexRepo',
    },
    {
      id: 'dependency',
      label: 'mix.exs',
      code: mixDependency(name, version, repository),
      hint: 'browse.usage.hint.hexDependency',
    },
    {
      id: 'rebar3',
      label: 'rebar.config',
      code: `${rebarRepository(repository, url, hosted)}\n{deps, [${rebarDependency(name, version)}]}.`,
      hint: 'browse.usage.hint.hexRebar',
    },
  ];
  if (hosted) {
    snippets.push({
      id: 'publish',
      label: 'mix hex.publish',
      labelKey: 'browse.usage.label.upload',
      code: `HEX_API_URL=${url}/api HEX_API_KEY=<access token> mix hex.publish`,
      hint: 'browse.usage.hint.hexPublish',
    });
  }
  return snippets;
}
