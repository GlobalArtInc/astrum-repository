import { describe, expect, it } from 'vitest';
import { usageSnippets } from './snippets';

const commit = 'f171d7baecaf37b5da5a3616d8833b9969753535';
const base = {
  format: 'huggingface',
  repositoryName: 'hf',
  repositoryType: 'proxy' as const,
  baseUrl: 'https://astrum.example.com',
};

describe('huggingface usage', () => {
  it('points HF_ENDPOINT at the repository and downloads the file at its commit', () => {
    const snippets = usageSnippets({
      ...base,
      component: { group: 'models', name: 'acme/bert', version: commit },
      path: `/models/acme/bert/${commit}/onnx/model.onnx`,
    });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['endpoint', 'cli', 'python', 'transformers']);
    expect(snippets[0]?.code).toBe(
      'export HF_ENDPOINT=https://astrum.example.com/repository/hf\nexport HF_HUB_DOWNLOAD_TIMEOUT=600',
    );
    expect(snippets[1]?.code).toBe(`hf download acme/bert onnx/model.onnx --revision ${commit}`);
    expect(snippets[2]?.code).toContain('hf_hub_download(');
    expect(snippets[2]?.code).toContain('filename="onnx/model.onnx"');
  });

  it('downloads whole datasets and only names the endpoint above a commit', () => {
    const dataset = usageSnippets({
      ...base,
      component: { group: 'datasets', name: 'acme/set', version: commit },
    });
    expect(dataset.map((snippet) => snippet.id)).toEqual(['endpoint', 'cli', 'python']);
    expect(dataset[1]?.code).toBe(`hf download acme/set --repo-type dataset --revision ${commit}`);
    expect(dataset[2]?.code).toContain('snapshot_download(');
    expect(usageSnippets({ ...base, path: '/models/acme/' }).map((snippet) => snippet.id)).toEqual([
      'endpoint',
    ]);
  });

  it('names the file the asset details carry for the repository', () => {
    const snippets = usageSnippets({
      ...base,
      component: { group: 'spaces', name: 'acme/demo', version: commit },
      attributes: { file: 'app.py', revisions: ['main'] },
    });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['endpoint', 'cli', 'python']);
    expect(snippets[1]?.code).toBe(`hf download acme/demo app.py --repo-type space --revision ${commit}`);
    expect(snippets[2]?.code).toContain('filename="app.py",\n    repo_type="space",');
  });
});
