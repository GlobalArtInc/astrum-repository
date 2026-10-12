import { getUsageSnippets } from '..';
import type { UsageContext } from '../types';

const URL = 'https://astrum.example.com/repository/tf-hosted/';

function snippets(overrides: Partial<UsageContext>): Record<string, string> {
  const map: Record<string, string> = {};
  for (const snippet of getUsageSnippets({
    format: 'terraform',
    repositoryName: 'tf-hosted',
    repositoryUrl: URL,
    repositoryType: 'hosted',
    ...overrides,
  })) {
    map[snippet.id] = snippet.snippetText;
  }
  return map;
}

describe('terraform usage', () => {
  it('addresses a module through the registry host and uploads it to its path', () => {
    const s = snippets({ group: 'acme', name: 'vpc/aws', version: '1.2.0' });

    expect(Object.keys(s)).toEqual(['module', 'credentials', 'upload']);
    expect(s.module).toBe('module "vpc" {\n  source  = "astrum.example.com/acme/vpc/aws"\n  version = "1.2.0"\n}');
    expect(s.credentials).toBe(
      `# curl -u <user> ${URL}token\ncredentials "astrum.example.com" {\n  token = "<token>"\n}`,
    );
    expect(s.upload).toBe(
      `curl -u <user> --upload-file vpc-aws-1.2.0.tar.gz ${URL}v1/modules/acme/vpc/aws/1.2.0/vpc-aws-1.2.0.tar.gz`,
    );
  });

  it('requires a hosted provider through the registry or the network mirror', () => {
    const s = snippets({
      group: 'acme',
      name: 'demo',
      version: '2.0.0',
      path: 'v1/providers/acme/demo/2.0.0/terraform-provider-demo_2.0.0_linux_amd64.zip',
    });

    expect(Object.keys(s)).toEqual(['provider', 'mirror', 'credentials', 'upload']);
    expect(s.provider).toContain('demo = {\n      source  = "astrum.example.com/acme/demo"\n      version = "2.0.0"');
    expect(s.mirror).toBe(`provider_installation {\n  network_mirror {\n    url = "${URL}"\n  }\n}`);
    expect(s.upload).toBe(
      'for file in terraform-provider-demo_2.0.0_*; do\n' +
        `  curl -u <user> --upload-file "$file" ${URL}v1/providers/acme/demo/2.0.0/"$file"\ndone`,
    );
  });

  it('keeps the upstream address of a proxied provider', () => {
    const s = snippets({ repositoryType: 'proxy', group: 'hashicorp', name: 'null', version: '3.2.3' });

    expect(s.provider).toContain('source  = "hashicorp/null"');
    expect(s.upload).toBeUndefined();
  });

  it('reads the module from the path of an asset', () => {
    const s = snippets({ repositoryType: 'group', path: '/v1/modules/acme/network/aws/1.0.0/network-aws-1.0.0.tar.gz' });

    expect(s.module).toBe('module "network" {\n  source  = "astrum.example.com/acme/network/aws"\n}');
  });

  it('offers nothing for an address it cannot read', () => {
    expect(snippets({ name: 'tool' })).toEqual({});
    expect(snippets({ group: 'acme', name: 'a/b/c' })).toEqual({});
  });
});
