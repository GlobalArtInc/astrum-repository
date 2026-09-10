

import { useCallback, useEffect, useMemo } from 'react';
import { useMachine } from '@xstate/react';

import { ExtJS } from '../../../../../../interface/ExtJS';

import { createEndpointDetailMachine, type EndpointDetailTab } from './endpointDetailMachine';
import { canGrantAccess, canReadSecurityDirectory } from './utils/endpointPermissions';

export interface UseEndpointDetailReturn {
  tab: EndpointDetailTab;
  hasSecurityDirectoryRead: boolean;
  hasGrantAccess: boolean;
  selectTab: (tab: string) => void;
}

const KNOWN_TABS: readonly EndpointDetailTab[] = ['try', 'who', 'grant'];

function isKnownTab(value: string): value is EndpointDetailTab {
  return (KNOWN_TABS as readonly string[]).includes(value);
}

/**
 * ViewModel for {@link ../EndpointDetail}. Wraps {@link createEndpointDetailMachine} and
 * feeds it permission changes as they resolve so the machine owns the current-tab
 * selection and any auto-reset when the active tab loses visibility.
 */
export function useEndpointDetail(): UseEndpointDetailReturn {
  const hasUser = ExtJS.useUser() ?? false;
  const hasSecurityDirectoryRead = ExtJS.usePermission(() => canReadSecurityDirectory(), [hasUser]);
  const hasGrantAccess = ExtJS.usePermission(() => canGrantAccess(), [hasUser]);

  const machine = useMemo(
    () =>
      createEndpointDetailMachine({
        hasSecurityDirectoryRead,
        hasGrantAccess,
      }),
    // eslint-disable-next-line react-hooks/exhaustive-deps -- machine is created once; permission changes flow in via PERMISSIONS_UPDATED events below
    []
  );

  const [state, send] = useMachine(machine);

  useEffect(() => {
    send({ type: 'PERMISSIONS_UPDATED', hasSecurityDirectoryRead, hasGrantAccess });
  }, [hasSecurityDirectoryRead, hasGrantAccess, send]);

  const tab: EndpointDetailTab = state.matches('who') ? 'who' : state.matches('grant') ? 'grant' : 'try';

  const selectTab = useCallback(
    (nextTab: string) => {
      if (isKnownTab(nextTab)) {
        send({ type: 'SELECT_TAB', tab: nextTab });
      }
    },
    [send]
  );

  return {
    tab,
    hasSecurityDirectoryRead,
    hasGrantAccess,
    selectTab,
  };
}
