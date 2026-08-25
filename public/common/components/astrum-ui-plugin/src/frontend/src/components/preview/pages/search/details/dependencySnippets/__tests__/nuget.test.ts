

import { generate } from '../nuget';
import type { SnippetComponentModel } from '../types';

describe('nuget dependency snippet generator', () => {
  const component: SnippetComponentModel = {
    format: 'nuget',
    group: '',
    name: 'Newtonsoft.Json',
    version: '13.0.3',
    repositoryName: 'nuget-hosted',
  };

  it('emits the Classic tool set in order', () => {
    expect(generate(component).map((s) => s.displayName)).toEqual([
      'Package Manager',
      '.NET CLI',
      'Paket CLI',
      'Chocolatey CLI',
    ]);
  });

  it('produces snippet text matching Classic, sourcing the repo URL from origin', () => {
    const s: Record<string, string> = {};
    for (const g of generate(component)) s[g.displayName] = g.snippetText;
    expect(s['Package Manager']).toBe('Install-Package Newtonsoft.Json -Version 13.0.3');
    expect(s['.NET CLI']).toBe('dotnet add package Newtonsoft.Json --version 13.0.3');
    expect(s['Paket CLI']).toBe('paket add Newtonsoft.Json --version 13.0.3');
    expect(s['Chocolatey CLI']).toBe(
      `choco install Newtonsoft.Json --version 13.0.3 --source="${window.location.origin}/repository/nuget-hosted/"`,
    );
  });

  it('omits the --source flag when the repository name is unknown (no "undefined" in the URL)', () => {
    const noRepo: SnippetComponentModel = {
      format: 'nuget',
      group: '',
      name: 'Newtonsoft.Json',
      version: '13.0.3',
    };
    const s: Record<string, string> = {};
    for (const g of generate(noRepo)) s[g.displayName] = g.snippetText;
    expect(s['Chocolatey CLI']).toBe('choco install Newtonsoft.Json --version 13.0.3');
    expect(s['Chocolatey CLI']).not.toContain('undefined');
  });
});
