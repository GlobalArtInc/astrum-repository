import { usageStrings } from './strings';
import { isHosted, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

function xml(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

/**
 * NuGet usage: the .NET CLI and nuget.exe against the repository's v3 service index, NuGet.Config
 * and, for a hosted repository, pushing a package.
 */
export const generate: UsageGenerator = (context) => {
  const { name, version, repositoryName } = context;
  if (!name) {
    return [];
  }
  const url = withSlash(context.repositoryUrl);
  const index = `${url}index.json`;
  const strings = usageStrings();
  const versionFlag = version ? ` --version ${version}` : '';

  const snippets: UsageSnippet[] = [
    { id: 'dotnet', label: '.NET CLI', snippetText: `dotnet add package ${name}${versionFlag} --source ${index}` },
    {
      id: 'source',
      label: 'dotnet nuget',
      snippetText: `dotnet nuget add source ${index} --name ${repositoryName}`,
      description: strings.HINTS.NUGET_SOURCE,
    },
    {
      id: 'nuget',
      label: 'nuget.exe',
      snippetText: `nuget sources add -Name ${repositoryName} -Source ${index}`,
      description: strings.HINTS.NUGET_SOURCE,
    },
    {
      id: 'config',
      label: 'NuGet.Config',
      snippetText:
        '<configuration>\n' +
        '  <packageSources>\n' +
        `    <add key="${xml(repositoryName)}" value="${xml(index)}" />\n` +
        '  </packageSources>\n' +
        '</configuration>',
      description: strings.HINTS.NUGET_CONFIG,
    },
  ];
  if (isHosted(context.repositoryType)) {
    const file = version ? `${name}.${version}.nupkg` : `${name}.nupkg`;
    snippets.push({
      id: 'push',
      label: 'dotnet nuget push',
      snippetText: `dotnet nuget push ${file} --source ${url} --api-key <api-key>`,
      description: strings.HINTS.NUGET_PUSH,
    });
  }
  return snippets;
};
