import { describe, expect, it } from 'vitest';
import { usageSnippets } from './snippets';

const base = {
  format: 'terraform',
  repositoryName: 'tf-hosted',
  repositoryType: 'hosted' as const,
  baseUrl: 'https://astrum.example.com',
};

describe('terraform usage', () => {
  it('shows the module block, the credentials and the upload of a module', () => {
    const snippets = usageSnippets({
      ...base,
      component: { group: 'acme', name: 'vpc/aws', version: '1.2.0' },
    });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['module', 'credentials', 'upload']);
    expect(snippets[0]?.code).toBe(
      'module "vpc" {\n  source  = "astrum.example.com/acme/vpc/aws"\n  version = "1.2.0"\n}',
    );
    expect(snippets[0]?.hintVars).toEqual({ host: 'astrum.example.com' });
    expect(snippets[1]?.code).toContain(
      'curl -u <user> https://astrum.example.com/repository/tf-hosted/token',
    );
    expect(snippets[2]?.code).toBe(
      'curl -u <user> --upload-file vpc-aws-1.2.0.tar.gz https://astrum.example.com/repository/tf-hosted/v1/modules/acme/vpc/aws/1.2.0/vpc-aws-1.2.0.tar.gz',
    );
  });

  it('reads providers from their path and offers the network mirror', () => {
    const snippets = usageSnippets({
      ...base,
      repositoryType: 'proxy',
      path: '/v1/providers/hashicorp/aws/5.0.0/terraform-provider-aws_5.0.0_linux_amd64.zip',
    });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['provider', 'mirror', 'credentials']);
    expect(snippets[0]?.code).toContain('source  = "hashicorp/aws"');
    expect(snippets[0]?.hint).toBe('browse.usage.hint.terraformProviderProxy');
    expect(snippets[1]?.code).toContain('url = "https://astrum.example.com/repository/tf-hosted/"');
  });

  it('has nothing to say without an address', () => {
    expect(usageSnippets({ ...base, path: '/.well-known/terraform.json' })).toEqual([]);
  });
});
