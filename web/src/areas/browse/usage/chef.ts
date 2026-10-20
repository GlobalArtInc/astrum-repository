import type { Snippet, UsageContext } from './snippets';

const PLACEHOLDER = '<cookbook>';

function cookbookLine(name: string, version: string): string {
  return version ? `cookbook "${name}", "= ${version}"` : `cookbook "${name}"`;
}

export function berksfile(url: string, name: string, version: string): string {
  return [`source "${url}"`, '', cookbookLine(name, version)].join('\n');
}

export function policyfile(url: string, name: string, version: string): string {
  return [
    `default_source :supermarket, "${url}"`,
    '',
    `run_list "${name}::default"`,
    cookbookLine(name, version),
  ].join('\n');
}

export function knifeDownload(url: string, name: string, version: string): string {
  return `knife supermarket download ${name}${version ? ` ${version}` : ''} -m ${url}`;
}

export function chef(context: UsageContext, repositoryUrl: string): Snippet[] {
  const url = repositoryUrl.replace(/\/+$/, '');
  const name = context.component?.name || PLACEHOLDER;
  const version = context.component?.version ?? '';
  const snippets: Snippet[] = [
    {
      id: 'berksfile',
      label: 'Berksfile',
      code: berksfile(url, name, version),
      hint: 'browse.usage.hint.chefBerksfile',
    },
    {
      id: 'policyfile',
      label: 'Policyfile.rb',
      code: policyfile(url, name, version),
      hint: 'browse.usage.hint.chefPolicyfile',
    },
    {
      id: 'knife',
      label: 'knife',
      code: knifeDownload(url, name, version),
      hint: 'browse.usage.hint.chefKnife',
    },
  ];
  if (context.repositoryType === 'hosted') {
    const folder = context.component?.name || '<name>';
    snippets.push({
      id: 'publish',
      label: 'curl',
      labelKey: 'browse.usage.label.upload',
      code: [
        `knife cookbook metadata from file ${folder}/metadata.rb`,
        `tar -czf ${folder}.tgz ${folder}`,
        `curl -u <user> -F 'cookbook={"category":"Other"}' -F tarball=@${folder}.tgz ${url}/api/v1/cookbooks`,
      ].join('\n'),
      hint: 'browse.usage.hint.chefPublish',
    });
  }
  return snippets;
}
