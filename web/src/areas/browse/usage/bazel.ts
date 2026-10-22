import type { Snippet, UsageContext } from './snippets';

const VERSION_FILE = /^\/?modules\/([^/]+)\/([^/]+)\/(?:MODULE\.bazel|source\.json|patches\/.+|overlay\/.+)$/;
const METADATA_FILE = /^\/?modules\/([^/]+)\/metadata\.json$/;
const CENTRAL_REGISTRY = 'https://bcr.bazel.build';

export function moduleOf(context: UsageContext): { name: string; version: string } {
  const component = context.component;
  if (component?.name) {
    return { name: component.name, version: component.version ?? '' };
  }
  const path = context.path ?? '';
  const file = VERSION_FILE.exec(path);
  if (file) {
    return { name: file[1] ?? '', version: file[2] ?? '' };
  }
  const metadata = METADATA_FILE.exec(path);
  return { name: metadata?.[1] ?? '', version: '' };
}

export function registryLine(url: string): string {
  return `common --registry=${url}`;
}

export function bazelDep(name: string, version: string): string {
  return `bazel_dep(name = "${name || '<module>'}", version = "${version || '<version>'}")`;
}

function hostOf(url: string): string {
  try {
    return new URL(url).hostname;
  } catch {
    return '<host>';
  }
}

export function bazel(context: UsageContext, repositoryUrl: string): Snippet[] {
  const url = repositoryUrl.replace(/\/+$/, '');
  const { name, version } = moduleOf(context);
  const hosted = context.repositoryType === 'hosted';
  const snippets: Snippet[] = [
    {
      id: 'registry',
      label: '.bazelrc',
      code: [registryLine(url), ...(hosted ? [registryLine(CENTRAL_REGISTRY)] : [])].join('\n'),
      hint: hosted ? 'browse.usage.hint.bazelRegistryHosted' : 'browse.usage.hint.bazelRegistry',
    },
    {
      id: 'netrc',
      label: '~/.netrc',
      code: `machine ${hostOf(url)} login <user> password <access token>`,
      hint: 'browse.usage.hint.bazelNetrc',
    },
    {
      id: 'dependency',
      label: 'MODULE.bazel',
      code: bazelDep(name, version),
      hint: 'browse.usage.hint.bazelDependency',
    },
  ];
  if (hosted) {
    const base = `${url}/modules/${name || '<module>'}`;
    snippets.push(
      {
        id: 'upload',
        label: 'curl',
        labelKey: 'browse.usage.label.upload',
        code: [
          `curl -u <user> -T MODULE.bazel ${base}/${version || '<version>'}/MODULE.bazel`,
          `curl -u <user> -T source.json ${base}/${version || '<version>'}/source.json`,
        ].join('\n'),
        hint: 'browse.usage.hint.bazelUpload',
      },
      {
        id: 'metadata',
        label: 'metadata.json',
        code: `curl -u <user> -T metadata.json ${base}/metadata.json`,
        hint: 'browse.usage.hint.bazelMetadata',
      },
    );
  }
  return snippets;
}
