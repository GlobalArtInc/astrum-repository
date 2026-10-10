

import Axios from 'axios';

import { normalizeFormatKey } from './registry';

/**
 * Fire the dependency-snippet copy analytics event.
 *
 * Mirrors Classic AS.analytics.controller.Analytics#copySnippetClicked: a fire-and-forget
 * POST to `/service/rest/dependency_snippets` with the normalized format key and the copied
 * snippet's display name. Failures are swallowed so analytics never disrupts the copy action.
 */
export function trackSnippetCopy(format: string, snippetDisplayName: string): void {
  const formatKey = normalizeFormatKey(format);
  const url =
    `/service/rest/dependency_snippets?format=${encodeURIComponent(formatKey)}` +
    `&snippet=${encodeURIComponent(snippetDisplayName)}`;
  Axios.post(url).catch(() => {
    // fire-and-forget: analytics failures must not affect the copy UX
  });
}
