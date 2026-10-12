import { usageStrings } from './strings';
import { hostOf, isHosted, withSlash } from './urls';
import type { UsageContext, UsageGenerator, UsageSnippet } from './types';

/**
 * The module (`namespace/name/system`) or provider (`namespace/type`) a component or asset stands for, from the
 * asset path when there is one, else from the component: its group is the namespace, its name `name/system` for a
 * module and the type for a provider.
 */
function addressOf(context: UsageContext): { kind: 'module' | 'provider'; parts: string[] } | null {
  const path = (context.path ?? '').replace(/^\/+/, '');
  const fromPath = /^v1\/(modules|providers)\/(.+)$/.exec(path);
  if (fromPath) {
    const segments = fromPath[2].split('/');
    return fromPath[1] === 'modules'
      ? { kind: 'module', parts: segments.slice(0, 3) }
      : { kind: 'provider', parts: segments.slice(0, 2) };
  }
  const parts = (context.name ?? '').split('/').filter(Boolean);
  if (!context.group) {
    return null;
  }
  if (parts.length === 2) {
    return { kind: 'module', parts: [context.group, ...parts] };
  }
  if (parts.length === 1) {
    return { kind: 'provider', parts: [context.group, parts[0]] };
  }
  return null;
}

/**
 * Terraform usage: the module block or provider requirement, the CLI configuration with the bearer token of this
 * server, the network mirror for providers and, for a hosted repository, uploading a release.
 */
export const generate: UsageGenerator = (context) => {
  const address = addressOf(context);
  if (!address || address.parts.length < (address.kind === 'module' ? 3 : 2)) {
    return [];
  }
  const url = withSlash(context.repositoryUrl);
  const host = hostOf(context.repositoryUrl);
  const strings = usageStrings();
  const version = context.version ?? '';
  const versionLine = version ? `\n  version = "${version}"` : '';
  const credentials = {
    id: 'credentials',
    label: '.terraformrc',
    snippetText:
      `# curl -u <user> ${url}token\n` +
      `credentials "${host}" {\n  token = "<token>"\n}`,
    description: strings.HINTS.TERRAFORM_CREDENTIALS,
  };
  const snippets: UsageSnippet[] = [];

  if (address.kind === 'module') {
    const [namespace, name, system] = address.parts;
    snippets.push({
      id: 'module',
      label: 'main.tf',
      snippetText: `module "${name}" {\n  source  = "${host}/${namespace}/${name}/${system}"${versionLine}\n}`,
      description: strings.HINTS.TERRAFORM_REGISTRY(host),
    });
    snippets.push(credentials);
    if (isHosted(context.repositoryType) && version) {
      const file = `${name}-${system}-${version}.tar.gz`;
      snippets.push({
        id: 'upload',
        label: strings.LABELS.UPLOAD,
        snippetText: `curl -u <user> --upload-file ${file} ${url}v1/modules/${namespace}/${name}/${system}/${version}/${file}`,
        description: strings.HINTS.UPLOAD_WRITE_ACCESS,
      });
    }
    return snippets;
  }

  const [namespace, type] = address.parts;
  const proxy = context.repositoryType === 'proxy';
  const source = proxy ? `${namespace}/${type}` : `${host}/${namespace}/${type}`;
  snippets.push({
    id: 'provider',
    label: 'main.tf',
    snippetText:
      'terraform {\n  required_providers {\n' +
      `    ${type} = {\n      source  = "${source}"${version ? `\n      version = "${version}"` : ''}\n    }\n` +
      '  }\n}',
    description: proxy ? strings.HINTS.TERRAFORM_PROVIDER_PROXY : strings.HINTS.TERRAFORM_REGISTRY(host),
  });
  snippets.push({
    id: 'mirror',
    label: 'network_mirror',
    snippetText: `provider_installation {\n  network_mirror {\n    url = "${url}"\n  }\n}`,
    description: strings.HINTS.TERRAFORM_MIRROR,
  });
  snippets.push(credentials);
  if (isHosted(context.repositoryType) && version) {
    snippets.push({
      id: 'upload',
      label: strings.LABELS.UPLOAD,
      snippetText:
        `for file in terraform-provider-${type}_${version}_*; do\n` +
        `  curl -u <user> --upload-file "$file" ${url}v1/providers/${namespace}/${type}/${version}/"$file"\n` +
        'done',
      description: strings.HINTS.UPLOAD_WRITE_ACCESS,
    });
  }
  return snippets;
};
