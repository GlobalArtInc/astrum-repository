

import { useCallback, useEffect, useMemo, useRef } from 'react';
import { useMachine } from '@xstate/react';

import { restClient, ENDPOINTS } from '../../../../../interface/api';
import { isMockMode } from '../../../config/featureFlags';
import { parseGaId } from './gaIdUtils';
import {
  createGaRepositoriesMachine,
  type RepoRow,
  type VersionResult,
} from './gaRepositoriesMachine';

interface Options {
  gaId: string;
  selectedVersion: string | null;
}

export interface UseGARepositoriesForVersionResult {
  rows: readonly RepoRow[];
  totalCount: number;
  loading: boolean;
  error: string | null;
  refresh: () => void;
}

/** Stable identity for the no-result case — see the note on `rows` in the return below. */
const EMPTY_ROWS: readonly RepoRow[] = [];

const MOCK_FALLBACK: VersionResult = {
  items: [
    { repositoryName: 'mock-hosted', type: 'hosted', versionCount: 2 },
    { repositoryName: 'mock-proxy',  type: 'proxy',  versionCount: 5 },
  ],
  totalCount: 2,
};

export function useGARepositoriesForVersion({
  gaId,
  selectedVersion,
}: Options): UseGARepositoriesForVersionResult {
  // Machine identity stable across renders per (gaId) — GA_CHANGED handles inter-machine transitions.
  const machine = useMemo(() => createGaRepositoriesMachine(gaId), []); // eslint-disable-line react-hooks/exhaustive-deps

  const [state, send] = useMachine(machine, {
    services: {
      fetchForVersion: async (ctx) => {
        if (ctx.selectedVersion === null) {
          return { items: [], totalCount: 0 };
        }
        const { format, group, name } = parseGaId(ctx.gaId);
        // Bare-name gaIds cannot address a component on the backend — skip the fetch.
        if (!format || !name) {
          return { items: [], totalCount: 0 };
        }
        const params = new URLSearchParams({
          format,
          namespace: group,
          name,
          version: ctx.selectedVersion,
        });
        const url = `${ENDPOINTS.SEARCH_REPOSITORIES}?${params.toString()}`;
        try {
          return await restClient.get<VersionResult>(url);
        }
        catch (err) {
          if (isMockMode()) {
            // eslint-disable-next-line no-console
            console.warn('[useGARepositoriesForVersion] fetch failed, using MOCK_FALLBACK:', err);
            return MOCK_FALLBACK;
          }
          throw err;
        }
      },
    },
  });

  // Track last dispatched values to prevent duplicate events.
  const lastDispatchedRef = useRef<{
    gaId: string;
    version: string | null;
  }>({ gaId, version: null });

  useEffect(() => {
    const last = lastDispatchedRef.current;

    if (last.gaId !== gaId) {
      // gaId changed: invalidate cache and re-select version.
      last.gaId = gaId;
      last.version = selectedVersion;
      send({ type: 'GA_CHANGED', gaId });
      if (selectedVersion !== null) {
        send({ type: 'SELECT_VERSION', version: selectedVersion });
      }
    } else if (last.version !== selectedVersion) {
      // Only version changed.
      last.version = selectedVersion;
      send({ type: 'SELECT_VERSION', version: selectedVersion });
    }
  }, [gaId, selectedVersion, send]); // eslint-disable-line react-hooks/exhaustive-deps

  const refresh = useCallback(() => {
    send({ type: 'REFRESH' });
  }, [send]);

  return {
    // Shared constant, not a fresh `[]`: while currentResult is null this hook returned a new
    // array identity on every render, so every consumer memo keyed on `rows` recomputed and every
    // derived prop changed identity each render. A versionless component sits in that state
    // permanently (ASTRUM-54201).
    rows: state.context.currentResult?.items ?? EMPTY_ROWS,
    totalCount: state.context.currentResult?.totalCount ?? 0,
    loading: state.matches('loading'),
    error: state.context.error,
    refresh,
  };
}
