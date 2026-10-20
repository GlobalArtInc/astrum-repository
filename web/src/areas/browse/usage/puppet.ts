import type { Snippet, UsageComponent, UsageContext } from './snippets';

const ARCHIVE = /^[A-Za-z0-9]+-[a-z][a-z0-9_]*-.+\.tar\.gz$/;

export function forgeUrl(repositoryUrl: string): string {
  return repositoryUrl.replace(/\/+$/, '');
}

export function moduleSlug(component: UsageComponent | undefined): string | null {
  if (!component?.name) {
    return null;
  }
  return component.group ? `${component.group}-${component.name}` : component.name;
}

export function installCommand(url: string, slug: string, version: string): string {
  return `puppet module install ${slug}${version ? ` --version ${version}` : ''} --module_repository ${url}`;
}

export function puppetfileLine(slug: string, version: string): string {
  return version ? `mod '${slug}', '${version}'` : `mod '${slug}'`;
}

function archiveName(context: UsageContext): string {
  const last = (context.path ?? '').split('/').pop() ?? '';
  if (ARCHIVE.test(last)) {
    return last;
  }
  const component = context.component;
  if (component?.name && component.group && component.version) {
    return `${component.group}-${component.name}-${component.version}.tar.gz`;
  }
  return '<owner>-<name>-<version>.tar.gz';
}

export function puppet(context: UsageContext, repositoryUrl: string): Snippet[] {
  const url = forgeUrl(repositoryUrl);
  const slug = moduleSlug(context.component) ?? '<owner>-<name>';
  const version = context.component?.version ?? '';
  const snippets: Snippet[] = [
    {
      id: 'config',
      label: 'puppet.conf',
      code: ['[main]', `module_repository = ${url}`, 'forge_authorization = <access token>'].join('\n'),
      hint: 'browse.usage.hint.puppetConfig',
    },
    {
      id: 'install',
      label: 'puppet module install',
      code: installCommand(url, slug, version),
    },
    {
      id: 'r10k',
      label: 'r10k.yaml',
      code: ['forge:', `  baseurl: '${url}'`, "  authorization_token: 'Bearer <access token>'"].join('\n'),
      hint: 'browse.usage.hint.puppetR10k',
    },
    {
      id: 'bolt',
      label: 'bolt-project.yaml',
      code: [
        'module-install:',
        '  forge:',
        `    baseurl: ${url}`,
        '    authorization_token: Bearer <access token>',
      ].join('\n'),
      hint: 'browse.usage.hint.puppetBolt',
    },
    {
      id: 'puppetfile',
      label: 'Puppetfile',
      code: [`forge '${url}'`, '', puppetfileLine(slug, version)].join('\n'),
      hint: 'browse.usage.hint.puppetfile',
    },
  ];
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'publish',
      label: 'pdk release publish',
      labelKey: 'browse.usage.label.upload',
      code: [
        'pdk build',
        `pdk release publish --forge-upload-url ${url}/v3/releases --forge-token <access token>`,
      ].join('\n'),
      hint: 'browse.usage.hint.puppetPublish',
    });
    snippets.push({
      id: 'curl',
      label: 'curl',
      code: `curl -H "Authorization: Bearer <access token>" -F file=@pkg/${archiveName(context)} ${url}/v3/releases`,
      hint: 'browse.usage.hint.puppetCurl',
    });
  }
  return snippets;
}
