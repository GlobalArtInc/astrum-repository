import { describe, expect, it } from 'vitest';
import { componentSnippets } from '../component/snippets';
import { installableUnit } from './p2';
import { supportsUsage, usageSnippets } from './snippets';

const base = {
  format: 'p2',
  repositoryName: 'eclipse',
  repositoryType: 'proxy' as const,
  baseUrl: 'https://astrum.example.com',
};
const url = 'https://astrum.example.com/repository/eclipse/';

describe('p2 usage', () => {
  it('shows Eclipse, Tycho and the p2 director with the repository address', () => {
    expect(supportsUsage('p2')).toBe(true);
    const snippets = usageSnippets(base);
    expect(snippets.map((snippet) => snippet.id)).toEqual(['eclipse', 'tycho', 'director']);
    expect(snippets[0]?.code).toBe(
      `Help > Install New Software... > Add...\nName: eclipse\nLocation: ${url}`,
    );
    expect(snippets[1]?.code).toBe(
      `<repositories>\n  <repository>\n    <id>eclipse</id>\n    <layout>p2</layout>\n    <url>${url}</url>\n  </repository>\n</repositories>`,
    );
    expect(snippets[2]?.code).toBe(
      `eclipse -nosplash -application org.eclipse.equinox.p2.director \\\n  -repository ${url} \\\n  -installIU <installable unit>`,
    );
    expect(snippets.map((snippet) => snippet.hint)).toEqual([
      'browse.usage.hint.p2Eclipse',
      'browse.usage.hint.p2Tycho',
      'browse.usage.hint.p2Director',
    ]);
  });

  it('installs the feature group of a feature and the plugin itself', () => {
    const feature = usageSnippets({
      ...base,
      component: { group: 'features', name: 'org.example.feature', version: '1.0.0' },
    });
    expect(feature[2]?.code).toContain('-installIU org.example.feature.feature.group/1.0.0');
    const plugin = usageSnippets({
      ...base,
      component: { group: 'plugins', name: 'org.example.core', version: '1.2.0.v2024' },
    });
    expect(plugin[2]?.code).toContain('-installIU org.example.core/1.2.0.v2024');
    expect(installableUnit(undefined)).toBeNull();
    expect(installableUnit({ group: 'binary', name: 'org.example.launcher', version: '1' })).toBe(
      'org.example.launcher',
    );
  });

  it('escapes the repository name in the Tycho snippet', () => {
    const snippets = usageSnippets({ ...base, repositoryName: 'a&b' });
    expect(snippets[1]?.code).toContain('<id>a&amp;b</id>');
    expect(snippets[1]?.code).toContain('/repository/a%26b/</url>');
  });

  it('gives the component page the director command and a target platform unit', () => {
    const snippets = componentSnippets({
      format: 'p2',
      group: 'features',
      name: 'org.example.feature',
      version: '1.0.0',
      baseUrl: 'https://astrum.example.com',
      repository: 'eclipse',
    });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['director', 'target']);
    expect(snippets[0]?.code).toBe(
      `eclipse -nosplash -application org.eclipse.equinox.p2.director \\\n  -repository ${url} \\\n  -installIU org.example.feature.feature.group/1.0.0`,
    );
    expect(snippets[1]?.code).toBe('<unit id="org.example.feature.feature.group" version="1.0.0"/>');
    expect(snippets[1]?.hint).toBe('browse.component.snippet.p2Target');
  });
});
