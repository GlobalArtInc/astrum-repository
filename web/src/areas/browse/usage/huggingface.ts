import type { Snippet, UsageContext } from './snippets';

const REPO_TYPES: Readonly<Record<string, string>> = {
  models: 'model',
  datasets: 'dataset',
  spaces: 'space',
};

function fileOf(context: UsageContext): string | null {
  const stored = context.attributes?.file;
  if (typeof stored === 'string' && stored) {
    return stored;
  }
  const component = context.component;
  if (!context.path || !component?.group || !component.name || !component.version) {
    return null;
  }
  const prefix = `${component.group}/${component.name}/${component.version}/`;
  const path = context.path.replace(/^\/+/, '');
  return path.startsWith(prefix) && path.length > prefix.length ? path.slice(prefix.length) : null;
}

export function huggingface(context: UsageContext, url: string): Snippet[] {
  const endpoint = url.replace(/\/+$/, '');
  const snippets: Snippet[] = [
    {
      id: 'endpoint',
      label: 'HF_ENDPOINT',
      code: `export HF_ENDPOINT=${endpoint}\nexport HF_HUB_DOWNLOAD_TIMEOUT=600`,
      hint: 'browse.usage.hint.hfEndpoint',
    },
  ];
  const component = context.component;
  if (!component?.name || !component.version) {
    return snippets;
  }
  const repoId = component.name;
  const revision = component.version;
  const repoType = REPO_TYPES[component.group] ?? 'model';
  const file = fileOf(context);
  const flags = `${repoType === 'model' ? '' : ` --repo-type ${repoType}`} --revision ${revision}`;
  snippets.push({
    id: 'cli',
    label: 'hf download',
    code: `hf download ${repoId}${file ? ` ${file}` : ''}${flags}`,
    hint: 'browse.usage.hint.hfDownload',
  });
  const download = file ? 'hf_hub_download' : 'snapshot_download';
  const argumentsList = [
    `    repo_id="${repoId}",`,
    ...(file ? [`    filename="${file}",`] : []),
    ...(repoType === 'model' ? [] : [`    repo_type="${repoType}",`]),
    `    revision="${revision}",`,
  ].join('\n');
  snippets.push({
    id: 'python',
    label: 'huggingface_hub',
    code: `from huggingface_hub import ${download}\n\npath = ${download}(\n${argumentsList}\n)`,
  });
  if (repoType === 'model') {
    snippets.push({
      id: 'transformers',
      label: 'transformers',
      code: `from transformers import AutoModel\n\nmodel = AutoModel.from_pretrained("${repoId}", revision="${revision}")`,
      hint: 'browse.usage.hint.hfTransformers',
    });
  }
  return snippets;
}
