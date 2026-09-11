

import { useCallback, useMemo } from 'react';
import { useMachine } from '@xstate/react';
import { createNodesMachine } from './nodesMachine';
import { NodeInfo } from './types';

export interface UseNodesReturn {
  nodes: NodeInfo[];
  loading: boolean;
  error: string | null;
  refresh: () => void;
  retry: () => void;
}

/**
 * Integration hook for the Nodes list. Wraps the nodesMachine and exposes a
 * simple UI-facing contract; no XState types leak to the component.
 */
export function useNodes(): UseNodesReturn {
  const machine = useMemo(() => createNodesMachine(), []);
  const [state, send] = useMachine(machine);

  const refresh = useCallback(() => send({ type: 'REFRESH' }), [send]);
  const retry = useCallback(() => send({ type: 'RETRY' }), [send]);

  return {
    nodes: state.context.nodes,
    loading: state.matches('loading'),
    error: state.context.loadError,
    refresh,
    retry,
  };
}
