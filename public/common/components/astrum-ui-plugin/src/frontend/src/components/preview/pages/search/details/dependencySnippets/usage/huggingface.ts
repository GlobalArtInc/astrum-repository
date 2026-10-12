import { usageStrings } from './strings';
import { withoutSlash } from './urls';
import type { UsageContext, UsageGenerator, UsageSnippet } from './types';

const REPO_TYPES: Readonly<Record<string, string>> = {
  models: 'model',
  datasets: 'dataset',
  spaces: 'space',
};

/**
 * The file of the repository the asset holds: its path after `<type>/<repo id>/<commit>/`.
 */
function fileOf(context: UsageContext): string | null {
  const stored = (context.attributes?.huggingface as Record<string, unknown> | undefined)?.file;
  if (typeof stored === 'string' && stored) {
    return stored;
  }
  if (!context.path || !context.group || !context.name || !context.version) {
    return null;
  }
  const prefix = `${context.group}/${context.name}/${context.version}/`;
  const path = context.path.replace(/^\/+/, '');
  return path.startsWith(prefix) && path.length > prefix.length ? path.slice(prefix.length) : null;
}

/**
 * Hugging Face usage: point huggingface_hub at the repository with HF_ENDPOINT, then download a file or the whole
 * repository at the commit of the component with the hf command, from Python or, for models, with transformers.
 * Folders above a commit carry no repository id, so they only get HF_ENDPOINT.
 */
export const generate: UsageGenerator = (context) => {
  const endpoint = withoutSlash(context.repositoryUrl);
  const strings = usageStrings();
  const snippets: UsageSnippet[] = [
    {
      id: 'endpoint',
      label: 'HF_ENDPOINT',
      snippetText: `export HF_ENDPOINT=${endpoint}`,
      description: strings.HINTS.HF_ENDPOINT,
    },
  ];
  const repoId = context.name;
  const revision = context.version;
  if (!repoId || !revision) {
    return snippets;
  }
  const repoType = REPO_TYPES[context.group ?? ''] ?? 'model';
  const file = fileOf(context);

  const cliFlags = `${repoType === 'model' ? '' : ` --repo-type ${repoType}`} --revision ${revision}`;
  snippets.push({
    id: 'cli',
    label: 'hf download',
    snippetText: `hf download ${repoId}${file ? ` ${file}` : ''}${cliFlags}`,
    description: strings.HINTS.HF_DOWNLOAD,
  });

  const pythonArguments = [
    `    repo_id="${repoId}",`,
    ...(file ? [`    filename="${file}",`] : []),
    ...(repoType === 'model' ? [] : [`    repo_type="${repoType}",`]),
    `    revision="${revision}",`,
  ].join('\n');
  const download = file ? 'hf_hub_download' : 'snapshot_download';
  snippets.push({
    id: 'python',
    label: 'huggingface_hub',
    snippetText: `from huggingface_hub import ${download}\n\npath = ${download}(\n${pythonArguments}\n)`,
  });

  if (repoType === 'model') {
    snippets.push({
      id: 'transformers',
      label: 'transformers',
      snippetText:
        'from transformers import AutoModel\n\n' +
        `model = AutoModel.from_pretrained("${repoId}", revision="${revision}")`,
      description: strings.HINTS.HF_TRANSFORMERS,
    });
  }
  return snippets;
};
