

import { useCallback, useEffect, useRef } from 'react';
import { useMachine } from '@xstate/react';
import { useCurrentStateAndParams, useRouter } from '@uirouter/react';
import { useToast } from '../../../../shared';
import { loggingConfigMachine } from './LoggingConfigMachine';
import LoggingStrings from '../../../../constants/pages/settings/support/LoggingStrings';

export type ViewMode = 'list' | 'create' | 'detail';

export interface UseLoggingConfigReturn {
  viewMode: ViewMode;
  selectedLogger: string | null;
  deleteDialogOpen: boolean;
  resetAllDialogOpen: boolean;
  isDeleting: boolean;
  isResettingAll: boolean;
  error: string | null;
  refreshKey: number;
  handleSelectLogger: (name: string) => void;
  handleCreate: () => void;
  handleBack: () => void;
  handleSave: () => void;
  handleDeleteClick: () => void;
  handleDeleteConfirm: () => void;
  handleCancelDelete: () => void;
  handleResetAll: () => void;
  handleResetAllConfirm: () => void;
  handleCancelResetAll: () => void;
  clearError: () => void;
}

const ROUTE_BASE = 'preview.admin.support.logging';

export function useLoggingConfig(): UseLoggingConfigReturn {
  const toast = useToast();
  const [state, send] = useMachine(loggingConfigMachine);
  const router = useRouter();
  const { state: routeState, params } = useCurrentStateAndParams();

  const prevStateRef = useRef(state);

  // The page has a route per view (…/logging, …/logging/create, …/logging/{logger}): follow the route, so a link or
  // the browser's back and forward buttons open the view, and change it when the user opens another view
  const routeName = routeState?.name ?? '';
  const routeView: ViewMode = routeName.endsWith('.create')
    ? 'create'
    : routeName.endsWith('.detail')
    ? 'detail'
    : 'list';
  const routeLogger: string | null = routeView === 'detail' ? params?.itemId ?? null : null;

  // biome-ignore lint/correctness/useExhaustiveDependencies: only a change of route drives the machine here; the machine's own changes navigate in the handlers
  useEffect(() => {
    if (routeView === 'create' && state.matches('list')) {
      send({ type: 'CREATE' });
    } else if (routeView === 'detail' && routeLogger && state.matches('list')) {
      send({ type: 'SELECT', name: routeLogger });
    } else if (routeView === 'list' && (state.matches('creating') || state.matches('editing'))) {
      send({ type: 'BACK' });
    }
  }, [routeView, routeLogger]);

  const goTo = useCallback((view: ViewMode, logger?: string) => {
    if (view === 'detail' && logger) {
      router.stateService.go(`${ROUTE_BASE}.detail`, { itemId: logger });
    } else {
      router.stateService.go(`${ROUTE_BASE}.${view === 'create' ? 'create' : 'list'}`);
    }
  }, [router]);

  // Show toast notifications on successful delete/reset operations.
  // `toast` is in deps so a reference change is picked up; extra runs are inert because
  // the prevState/state comparison only emits toasts on the specific transitions below.
  useEffect(() => {
    const prevState = prevStateRef.current;

    if (prevState.matches('deleting') && state.matches('list')) {
      // Read selectedLogger from prevState.context — by the time we reach 'list', the machine's
      // clearSelectedLogger action has already set it to null in the current state.
      const loggerName = prevState.context.selectedLogger;
      toast.success(LoggingStrings.HOOK.LOGGER_OVERRIDE_REMOVED(loggerName));
      goTo('list');
    }

    if (prevState.matches('resettingAll') && state.matches('list')) {
      toast.success(LoggingStrings.HOOK.ALL_LOGGERS_RESET_DEFAULT_LEVELS);
    }

    prevStateRef.current = state;
  }, [state, toast, goTo]);

  // Derive view mode from machine state
  const viewMode: ViewMode = state.matches('creating')
    ? 'create'
    : state.matches('editing') || state.matches('confirmDelete') || state.matches('deleting')
    ? 'detail'
    : 'list';

  const deleteDialogOpen = state.matches('confirmDelete') || state.matches('deleting');
  const resetAllDialogOpen = state.matches('confirmResetAll') || state.matches('resettingAll');
  const isDeleting = state.matches('deleting');
  const isResettingAll = state.matches('resettingAll');

  const handleSelectLogger = useCallback(
    (name: string) => {
      send({ type: 'SELECT', name });
      goTo('detail', name);
    },
    [send, goTo]
  );

  const handleCreate = useCallback(() => {
    send({ type: 'CREATE' });
    goTo('create');
  }, [send, goTo]);

  const handleBack = useCallback(() => {
    send({ type: 'BACK' });
    goTo('list');
  }, [send, goTo]);

  const handleSave = useCallback(() => {
    send({ type: 'SAVE' });
    goTo('list');
  }, [send, goTo]);

  const handleDeleteClick = useCallback(() => {
    send({ type: 'DELETE_CLICK' });
  }, [send]);

  const handleDeleteConfirm = useCallback(() => {
    send({ type: 'CONFIRM_DELETE' });
  }, [send]);

  const handleCancelDelete = useCallback(() => {
    send({ type: 'CANCEL_DELETE' });
  }, [send]);

  const handleResetAll = useCallback(() => {
    send({ type: 'RESET_ALL_CLICK' });
  }, [send]);

  const handleResetAllConfirm = useCallback(() => {
    send({ type: 'CONFIRM_RESET_ALL' });
  }, [send]);

  const handleCancelResetAll = useCallback(() => {
    send({ type: 'CANCEL_RESET_ALL' });
  }, [send]);

  const clearError = useCallback(() => {
    send({ type: 'CLEAR_ERROR' });
  }, [send]);

  return {
    viewMode,
    selectedLogger: state.context.selectedLogger,
    deleteDialogOpen,
    resetAllDialogOpen,
    isDeleting,
    isResettingAll,
    error: state.context.error,
    refreshKey: state.context.refreshKey,
    handleSelectLogger,
    handleCreate,
    handleBack,
    handleSave,
    handleDeleteClick,
    handleDeleteConfirm,
    handleCancelDelete,
    handleResetAll,
    handleResetAllConfirm,
    handleCancelResetAll,
    clearError,
  };
}
