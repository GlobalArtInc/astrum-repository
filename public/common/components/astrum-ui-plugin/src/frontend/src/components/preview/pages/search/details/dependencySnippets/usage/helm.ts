import { usageStrings } from './strings';
import { isHosted, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * Helm usage: add the repository, install the chart, declare it as a chart dependency and, for a
 * hosted repository, upload a packaged chart.
 */
export const generate: UsageGenerator = (context) => {
  const { name, version, repositoryName } = context;
  if (!name) {
    return [];
  }
  const url = withSlash(context.repositoryUrl);
  const strings = usageStrings();
  const versionFlag = version ? ` --version ${version}` : '';

  const snippets: UsageSnippet[] = [
    {
      id: 'repo',
      label: 'helm repo add',
      snippetText: `helm repo add ${repositoryName} ${url}\nhelm repo update`,
      description: strings.HINTS.HELM_REPO,
    },
    { id: 'install', label: 'helm install', snippetText: `helm install ${name} ${repositoryName}/${name}${versionFlag}` },
    {
      id: 'dependency',
      label: 'Chart.yaml',
      snippetText:
        'dependencies:\n' +
        `  - name: ${name}\n` +
        (version ? `    version: ${version}\n` : '') +
        `    repository: ${url}`,
    },
  ];
  if (isHosted(context.repositoryType) && version) {
    const file = `${name}-${version}.tgz`;
    snippets.push({
      id: 'upload',
      label: strings.LABELS.UPLOAD,
      snippetText: `curl -u <user> --upload-file ${file} ${url}${file}`,
      description: strings.HINTS.UPLOAD_WRITE_ACCESS,
    });
  }
  return snippets;
};
