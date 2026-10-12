import { useEffect, useState } from 'react';

import { restClient, ENDPOINTS } from '../../../../interface/api';
import { isMockMode } from '../../config/featureFlags';
import { MOCK_REPOSITORIES } from './mockData';
import type { RepositoryPageResponse, RepositoryReference } from './browse.types';

/**
 * What the browse page knows about the open repository.
 */
export interface RepositoryInfo extends RepositoryReference {
  /** APT distribution, read from the repository settings when the user may see them. */
  distribution?: string | null;
  /** Directory depth at which a Yum hosted repository generates repodata, when the user may see it. */
  repodataDepth?: number | null;
  /** Whether a Yum repository signs its repomd.xml, when the user may see its settings. */
  repodataSigned?: boolean | null;
}

/**
 * The repository settings the usage snippets read.
 */
interface UsageSettings {
  distribution: string | null;
  repodataDepth: number | null;
  repodataSigned: boolean | null;
}

const NO_SETTINGS: UsageSettings = { distribution: null, repodataDepth: null, repodataSigned: null };

/**
 * The absolute URL of a repository, from the server when it sent one, else built from this page's
 * origin. Always ends with a slash.
 */
export function repositoryUrlOf(name: string, url?: string | null): string {
  const base = url || (typeof window !== 'undefined' ? `${window.location.origin}/repository/${encodeURIComponent(name)}` : '');
  return `${base.replace(/\/+$/, '')}/`;
}

async function fetchRepository(name: string): Promise<RepositoryReference | null> {
  if (isMockMode()) {
    return MOCK_REPOSITORIES.find((repository) => repository.name === name) ?? null;
  }
  const response = await restClient.get<RepositoryPageResponse>(
    `${ENDPOINTS.REPOSITORIES_DETAILS}/filtered?nameFilter=${encodeURIComponent(name)}&pageSize=200`,
  );
  return (response?.data ?? []).find((repository) => repository.name === name) ?? null;
}

/**
 * Settings of the repository that the usage snippets need. Only administrators of the repository
 * may read them; for everyone else the snippets fall back to what they can tell without.
 */
async function fetchUsageSettings(name: string, format: string): Promise<UsageSettings> {
  if ((format !== 'apt' && format !== 'yum') || isMockMode()) {
    return NO_SETTINGS;
  }
  try {
    const settings = await restClient.get<{
      apt?: { distribution?: string };
      yum?: { repodataDepth?: number };
      yumSigning?: { keypair?: string };
    }>(`/service/rest/internal/ui/repositories/repository/${encodeURIComponent(name)}`);
    return {
      distribution: settings?.apt?.distribution ?? null,
      repodataDepth: settings?.yum?.repodataDepth ?? null,
      repodataSigned: format === 'yum' ? Boolean(settings?.yumSigning?.keypair) : null,
    };
  } catch {
    return NO_SETTINGS;
  }
}

/**
 * Loads the format, type, status and URL of a repository for the page header and the usage snippets.
 */
export function useRepositoryInfo(repositoryName: string | null | undefined): {
  repository: RepositoryInfo | null;
  loading: boolean;
} {
  const [state, setState] = useState<{ name: string | null; repository: RepositoryInfo | null }>({
    name: null,
    repository: null,
  });

  useEffect(() => {
    if (!repositoryName) {
      setState({ name: null, repository: null });
      return undefined;
    }
    let cancelled = false;
    fetchRepository(repositoryName)
      .then(async (found) => {
        if (!found) {
          return null;
        }
        const settings = await fetchUsageSettings(found.name, found.format);
        return { ...found, ...settings };
      })
      .catch(() => null)
      .then((info) => {
        if (!cancelled) {
          setState({ name: repositoryName, repository: info });
        }
      });
    return () => {
      cancelled = true;
    };
  }, [repositoryName]);

  const current = state.name === repositoryName;
  const repository = current ? state.repository : null;
  const loading = Boolean(repositoryName) && !current;
  return { repository, loading };
}
