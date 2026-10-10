

import type { SnippetGenerator } from './types';

/**
 * NuGet dependency snippet generator.
 *
 * Ported from Classic AS.nuget.controller.NuGetDependencySnippetController. The Chocolatey
 * snippet points at this Astrum instance's NuGet repository, built from the current origin.
 */
export const generate: SnippetGenerator = (component) => {
  const { name, version, repositoryName } = component;
  // Guard window so the generator is safe to call outside a browser (SSR/Node/tests).
  const origin = typeof window !== 'undefined' ? window.location.origin : '';
  // Only point Chocolatey at this instance's NuGet repo when we actually know its name;
  // otherwise emit the plain command rather than a `.../repository/undefined/` source URL.
  const source = repositoryName ? ` --source="${origin}/repository/${repositoryName}/"` : '';

  return [
    { displayName: 'Package Manager', snippetText: `Install-Package ${name} -Version ${version}` },
    { displayName: '.NET CLI', snippetText: `dotnet add package ${name} --version ${version}` },
    { displayName: 'Paket CLI', snippetText: `paket add ${name} --version ${version}` },
    {
      displayName: 'Chocolatey CLI',
      snippetText: `choco install ${name} --version ${version}${source}`,
    },
  ];
};
