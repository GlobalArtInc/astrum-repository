

/**
 * gaRepositoriesMachine — per-version cache and fetch lifecycle for the GA detail
 * Repositories tab. Owned above <Tabs.Content> in GADetailPage.tsx so the cache
 * survives tab switches (Radix unmounts inactive tab content).
 */

import { createMachine, assign } from 'xstate';
import S from '../../../constants/pages/search/ComponentDetailStrings';

export interface RepoRow {
  readonly repositoryName: string;
  readonly type: 'hosted' | 'proxy' | 'group';
  readonly versionCount: number;
}

export interface VersionResult {
  readonly items: readonly RepoRow[];
  readonly totalCount: number;
}

export interface GaRepositoriesMachineContext {
  gaId: string;
  selectedVersion: string | null;
  cache: Map<string, VersionResult>;
  currentResult: VersionResult | null;
  error: string | null;
}

export type GaRepositoriesMachineEvent =
  | { type: 'SELECT_VERSION'; version: string | null }
  | { type: 'GA_CHANGED'; gaId: string }
  | { type: 'REFRESH' };

/**
 * Factory. Callers pass in the current gaId; selectedVersion begins null and is
 * populated via SELECT_VERSION.
 */
export function createGaRepositoriesMachine(gaId: string) {
  return createMachine<GaRepositoriesMachineContext, GaRepositoriesMachineEvent>({
    id: 'gaRepositories',
    predictableActionArguments: true,
    initial: 'idle',
    context: {
      gaId,
      selectedVersion: null,
      cache: new Map(),
      currentResult: null,
      error: null,
    },
    states: {
      idle: {
        on: {
          SELECT_VERSION: [
            {
              cond: (_ctx, ev) => ev.version === null,
              actions: assign({ selectedVersion: (_c, e) => e.version, currentResult: () => null }),
            },
            {
              cond: (ctx, ev) => ev.version !== null && ctx.cache.has(ev.version),
              actions: assign({
                selectedVersion: (_c, e) => e.version,
                currentResult: (ctx, e) => ctx.cache.get(e.version as string) ?? null,
                error: () => null,
              }),
            },
            {
              target: 'loading',
              actions: assign({ selectedVersion: (_c, e) => e.version, error: () => null }),
            },
          ],
          GA_CHANGED: {
            actions: assign({
              gaId: (_c, e) => e.gaId,
              cache: () => new Map<string, VersionResult>(),
              currentResult: () => null,
              error: () => null,
              // selectedVersion intentionally preserved — the hook re-dispatches
              // SELECT_VERSION immediately after GA_CHANGED.
            }),
          },
          REFRESH: {
            target: 'loading',
            cond: (ctx) => ctx.selectedVersion !== null,
            actions: assign({
              cache: (ctx) => {
                const next = new Map(ctx.cache);
                if (ctx.selectedVersion !== null) next.delete(ctx.selectedVersion);
                return next;
              },
              currentResult: () => null,
              error: () => null,
            }),
          },
        },
      },

      loading: {
        invoke: {
          src: 'fetchForVersion',
          onDone: {
            target: 'idle',
            actions: assign({
              cache: (ctx, e) => {
                const next = new Map(ctx.cache);
                if (ctx.selectedVersion !== null) {
                  next.set(ctx.selectedVersion, e.data as VersionResult);
                }
                return next;
              },
              currentResult: (_c, e) => e.data as VersionResult,
              error: () => null,
            }),
          },
          onError: {
            target: 'error',
            actions: assign({
              error: (_c, e) => (e.data as Error)?.message ?? S.REPOSITORIES.LOAD_FAILED,
            }),
          },
        },
        on: {
          SELECT_VERSION: {
            target: 'loading',
            internal: false,
            actions: assign({ selectedVersion: (_c, e) => e.version, error: () => null }),
          },
          GA_CHANGED: {
            target: 'idle',
            actions: assign({
              gaId: (_c, e) => e.gaId,
              cache: () => new Map<string, VersionResult>(),
              currentResult: () => null,
              error: () => null,
            }),
          },
        },
      },

      error: {
        on: {
          SELECT_VERSION: {
            target: 'loading',
            actions: assign({ selectedVersion: (_c, e) => e.version, error: () => null }),
          },
          REFRESH: {
            target: 'loading',
            cond: (ctx) => ctx.selectedVersion !== null,
            // No cache eviction here: reaching the error state means the last fetch's
            // onDone never ran, so the current selectedVersion cannot have been written
            // to the cache. Asymmetry with the idle-state REFRESH is intentional.
          },
          GA_CHANGED: {
            target: 'idle',
            actions: assign({
              gaId: (_c, e) => e.gaId,
              cache: () => new Map<string, VersionResult>(),
              currentResult: () => null,
              error: () => null,
            }),
          },
        },
      },
    },
  });
}
