import { describe, expect, it } from 'vitest';
import { usageSnippets } from './snippets';

describe('swift usage', () => {
  it('sets the registry, the dependency, the sign-in and the publish command', () => {
    const snippets = usageSnippets({
      format: 'swift',
      repositoryName: 'swift-hosted',
      repositoryType: 'hosted',
      baseUrl: 'https://astrum.example.com',
      component: { group: 'mona', name: 'LinkedList', version: '1.1.1' },
    });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['registry', 'manifest', 'login', 'publish']);
    expect(snippets[0]?.code).toBe(
      'swift package-registry set https://astrum.example.com/repository/swift-hosted/',
    );
    expect(snippets[1]?.code).toBe('.package(id: "mona.LinkedList", from: "1.1.1")');
    expect(snippets[3]?.code).toBe(
      'swift package-registry publish mona.LinkedList 1.1.1 --url https://astrum.example.com/repository/swift-hosted/',
    );
  });

  it('allows plain HTTP and signs in through .netrc there', () => {
    const snippets = usageSnippets({
      format: 'swift',
      repositoryName: 'swift-group',
      repositoryType: 'group',
      baseUrl: 'http://localhost:8081',
    });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['registry', 'netrc']);
    expect(snippets[0]?.code).toContain('--allow-insecure-http');
    expect(snippets[1]?.code).toBe('machine localhost\n  login <user>\n  password <password>');
  });
});
