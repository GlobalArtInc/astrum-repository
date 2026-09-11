

import { assign, createMachine } from 'xstate';
import { fetchNodes } from './nodesApi';
import { NodeInfo } from './types';

export interface NodesContext {
  nodes: NodeInfo[];
  loadError: string | null;
}

type NodesEvent =
  | { type: 'RETRY' }
  | { type: 'REFRESH' }
  | { type: 'done.invoke.loadNodes'; data: NodeInfo[] }
  | { type: 'error.platform.loadNodes'; data: Error };

const NODES_LOAD_ERROR = 'Failed to load nodes';

/**
 * Read-only fetch machine for the cluster Nodes list.
 * loading -> loaded | error ; loaded handles REFRESH ; error handles RETRY.
 * The load service is invoked, so its result is discarded automatically on unmount.
 */
export function createNodesMachine() {
  return createMachine<NodesContext, NodesEvent>(
    {
      id: 'nodes',
      initial: 'loading',
      context: { nodes: [], loadError: null },
      states: {
        loading: {
          invoke: {
            id: 'loadNodes',
            src: 'load',
            onDone: { target: 'loaded', actions: 'setNodes' },
            onError: { target: 'error', actions: 'setError' },
          },
        },
        loaded: {
          on: { REFRESH: 'loading' },
        },
        error: {
          on: { RETRY: { target: 'loading', actions: 'clearError' } },
        },
      },
    },
    {
      actions: {
        setNodes: assign((_ctx, event) => ({
          nodes: ((event as any).data as NodeInfo[]) ?? [],
          loadError: null,
        })),
        setError: assign((_ctx, event) => ({
          // Clear any nodes from a prior successful load so the error state never
          // exposes stale data (e.g. a REFRESH that then fails).
          nodes: [],
          loadError: (event as any).data?.message ?? NODES_LOAD_ERROR,
        })),
        clearError: assign({ loadError: (_ctx: NodesContext) => null }),
      },
      services: {
        load: () => fetchNodes(),
      },
    },
  );
}
