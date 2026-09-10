

import { useEffect, useMemo } from 'react';
import { useMachine } from '@xstate/react';
import {
  createApiPageMachine,
  defaultLoadSwagger,
  type SwaggerSpec,
} from './apiPageMachine';

export interface UseApiPageOptions {
  swaggerUrl: string;
}

export interface UseApiPageReturn {
  swagger: SwaggerSpec | null;
  isLoading: boolean;
  hasError: boolean;
  refresh: () => void;
  retry: () => void;
}

/**
 * Hook wrapping the API-page state machine. Loads the swagger spec via the machine's
 * loading/loaded/loadError states and exposes flags plus REFRESH/RETRY handlers so the
 * page component stays presentation-only.
 */
export function useApiPage(options: UseApiPageOptions): UseApiPageReturn {
  const { swaggerUrl } = options;

  const machine = useMemo(
    () =>
      createApiPageMachine({
        swaggerUrl,
        loadSwagger: defaultLoadSwagger,
      }),
    // eslint-disable-next-line react-hooks/exhaustive-deps -- machine is created once with the initial swaggerUrl; URL changes are handled via the SET_SWAGGER_URL event in the effect below
    []
  );

  const [state, send] = useMachine(machine);

  useEffect(() => {
    send({ type: 'SET_SWAGGER_URL', url: swaggerUrl });
  }, [swaggerUrl, send]);

  return {
    swagger: state.context.swagger,
    isLoading: state.matches('loading'),
    hasError: state.matches('loadError'),
    refresh: () => send({ type: 'REFRESH' }),
    retry: () => send({ type: 'RETRY' }),
  };
}
