import { usageStrings } from './strings';
import { isHosted, withoutSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * Go usage: fetch the module through the repository as a GOPROXY, set it for good, require it in
 * go.mod and, for a hosted repository, upload a module zip. Modules published to a hosted
 * repository are not in the public checksum database, so it is skipped for them.
 */
export const generate: UsageGenerator = (context) => {
  const { name: module, version } = context;
  if (!module) {
    return [];
  }
  const proxy = withoutSlash(context.repositoryUrl);
  const strings = usageStrings();
  const hosted = isHosted(context.repositoryType);
  const spec = version ? `${module}@${version}` : module;

  const snippets: UsageSnippet[] = [
    { id: 'get', label: 'go get', snippetText: `GOPROXY=${proxy} go get ${spec}` },
    {
      id: 'env',
      label: 'go env',
      snippetText: `go env -w GOPROXY=${proxy},direct` + (hosted ? `\ngo env -w GONOSUMDB=${module}` : ''),
      description: hosted ? strings.HINTS.GO_ENV_HOSTED : strings.HINTS.GO_ENV,
    },
  ];
  if (version) {
    snippets.push({ id: 'mod', label: 'go.mod', snippetText: `require ${module} ${version}` });
  }
  if (hosted && version) {
    snippets.push({
      id: 'upload',
      label: strings.LABELS.UPLOAD,
      snippetText: `curl -u <user> --upload-file ${version}.zip ${proxy}/${module}/@v/${version}.zip`,
      description: strings.HINTS.UPLOAD_WRITE_ACCESS,
    });
  }
  return snippets;
};
