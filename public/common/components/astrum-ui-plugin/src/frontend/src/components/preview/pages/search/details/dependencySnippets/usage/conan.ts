import { usageStrings } from './strings';
import { isHosted, withSlash } from './urls';
import type { UsageGenerator, UsageSnippet } from './types';

/**
 * Conan usage: add the repository as a remote, install the package from it, conanfile.txt and, for a
 * hosted repository, uploading a package.
 */
export const generate: UsageGenerator = (context) => {
  const { name, version, repositoryName } = context;
  const url = withSlash(context.repositoryUrl);
  const strings = usageStrings();
  const reference = name && version ? `${name}/${version}` : name ?? '';

  const snippets: UsageSnippet[] = [
    {
      id: 'remote',
      label: 'conan remote',
      snippetText: `conan remote add ${repositoryName} ${url}`,
      description: strings.HINTS.CONAN_REMOTE,
    },
  ];
  if (reference) {
    snippets.push({ id: 'install', label: 'conan install', snippetText: `conan install --requires=${reference} -r ${repositoryName}` });
    snippets.push({ id: 'conanfile', label: 'conanfile.txt', snippetText: `[requires]\n${reference}` });
    if (isHosted(context.repositoryType)) {
      snippets.push({
        id: 'upload',
        label: 'conan upload',
        snippetText: `conan upload ${reference} -r ${repositoryName}`,
        description: strings.HINTS.UPLOAD_WRITE_ACCESS,
      });
    }
  }
  return snippets;
};
