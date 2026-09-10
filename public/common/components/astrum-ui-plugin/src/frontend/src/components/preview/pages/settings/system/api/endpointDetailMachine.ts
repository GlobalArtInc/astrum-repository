

import { assign, createMachine } from 'xstate';

export type EndpointDetailTab = 'try' | 'who' | 'grant';

export interface EndpointDetailContext {
  hasSecurityDirectoryRead: boolean;
  hasGrantAccess: boolean;
}

export type EndpointDetailEvent =
  | { type: 'SELECT_TAB'; tab: EndpointDetailTab }
  | { type: 'PERMISSIONS_UPDATED'; hasSecurityDirectoryRead: boolean; hasGrantAccess: boolean };

export interface CreateEndpointDetailMachineOptions {
  hasSecurityDirectoryRead: boolean;
  hasGrantAccess: boolean;
}

/**
 * Presentation-state machine for {@link ../EndpointDetail}. The active tab lives here so
 * the view can stay purely declarative; SELECT_TAB is guarded so callers cannot switch to
 * a hidden tab, and PERMISSIONS_UPDATED re-evaluates the current state so a revoked
 * permission bounces the user back to `try` without a race between render and effect.
 */
export function createEndpointDetailMachine(options: CreateEndpointDetailMachineOptions) {
  return createMachine<EndpointDetailContext, EndpointDetailEvent>(
    {
      id: 'endpoint-detail',
      predictableActionArguments: true,
      initial: 'try',
      context: {
        hasSecurityDirectoryRead: options.hasSecurityDirectoryRead,
        hasGrantAccess: options.hasGrantAccess,
      },
      on: {
        PERMISSIONS_UPDATED: { actions: 'assignPermissions' },
      },
      states: {
        try: {
          on: {
            SELECT_TAB: [
              { target: 'who', cond: 'canSelectWho' },
              { target: 'grant', cond: 'canSelectGrant' },
            ],
          },
        },
        who: {
          always: [{ target: 'try', cond: 'lostSecurityRead' }],
          on: {
            SELECT_TAB: [
              { target: 'try', cond: 'selectingTry' },
              { target: 'grant', cond: 'canSelectGrant' },
            ],
          },
        },
        grant: {
          always: [{ target: 'try', cond: 'lostGrantAccess' }],
          on: {
            SELECT_TAB: [
              { target: 'try', cond: 'selectingTry' },
              { target: 'who', cond: 'canSelectWho' },
            ],
          },
        },
      },
    },
    {
      actions: {
        assignPermissions: assign((_ctx, event) => {
          const e = event as Extract<EndpointDetailEvent, { type: 'PERMISSIONS_UPDATED' }>;
          return {
            hasSecurityDirectoryRead: e.hasSecurityDirectoryRead,
            hasGrantAccess: e.hasGrantAccess,
          };
        }),
      },
      guards: {
        selectingTry: (_ctx, event) =>
          (event as Extract<EndpointDetailEvent, { type: 'SELECT_TAB' }>).tab === 'try',
        canSelectWho: (ctx, event) =>
          (event as Extract<EndpointDetailEvent, { type: 'SELECT_TAB' }>).tab === 'who' &&
          ctx.hasSecurityDirectoryRead,
        canSelectGrant: (ctx, event) =>
          (event as Extract<EndpointDetailEvent, { type: 'SELECT_TAB' }>).tab === 'grant' &&
          ctx.hasGrantAccess,
        lostSecurityRead: (ctx) => !ctx.hasSecurityDirectoryRead,
        lostGrantAccess: (ctx) => !ctx.hasGrantAccess,
      },
    }
  );
}
