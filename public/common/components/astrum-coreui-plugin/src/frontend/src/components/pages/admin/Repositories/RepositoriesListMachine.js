

import Axios from 'axios';
import {assign} from 'xstate';
import {ListMachineUtils} from '@globalart/astrum-ui-plugin';
import {ExtAPIUtils, APIConstants} from '@globalart/astrum-ui-plugin';
import {mergeDeepRight, indexBy, prop} from 'ramda';

import {
  isIqServerEnabled,
  canReadFirewallStatus,
  canUpdateHealthCheck,
  hasFirewall
} from './IQServerColumns/IQServerHelpers';

const {
  EXT,
  REST: {INTERNAL}
} = APIConstants;

export default ListMachineUtils.buildListMachine({
  id: 'RepositoriesListMachine',
  sortableFields: ['name', 'type', 'format', 'status'],
  initial: 'loaded',

  config: (config) =>
    mergeDeepRight(config, {
      states: {
        loading: {
          states: {
            fetch: {
              invoke: {
                onDone: [
                  {
                    target: '#RepositoriesListMachine.readingHealthCheck',
                    cond: 'shouldRequestHealthCheck',
                    actions: ['clearError', 'setData']
                  },
                  {
                    target: '#RepositoriesListMachine.readingFirewallStatus',
                    cond: 'shouldRequestFirewallStatus',
                    actions: ['clearError', 'setData']
                  },
                  {
                    target: '#loaded',
                    actions: ['clearError', 'setData']
                  }
                ]
              }
            }
          }
        },
        loaded: {
          on: {
            ENABLE_HELTH_CHECK_SINGLE_REPO: {
              target: 'enablingHealthCheckSingleRepo'
            },
            ENABLE_HELTH_CHECK_ALL_REPOS: {
              target: 'enablingHealthCheckAllRepos'
            }
          }
        },
        readingHealthCheck: {
          invoke: {
            src: 'readHealthCheck',
            onDone: [
              {
                target: '#RepositoriesListMachine.readingFirewallStatus',
                cond: 'shouldRequestFirewallStatus',
                actions: ['setHealthCheck']
              },
              {
                target: 'loaded',
                actions: ['setHealthCheck']
              }
            ],
            onError: [
              {
                target: '#RepositoriesListMachine.readingFirewallStatus',
                cond: 'shouldRequestFirewallStatus',
                actions: 'setReadHealthCheckError'
              },
              {
                target: 'loaded',
                actions: ['setReadHealthCheckError']
              }
            ]
          },
          entry: 'clearReadHealthCheckError'
        },
        readingFirewallStatus: {
          invoke: {
            src: 'readFirewallRepositoryStatus',
            onDone: {
              target: 'loaded',
              actions: ['setFirewallRepositoryStatus']
            },
            onError: {
              target: 'loaded',
              actions: 'setFirewallRepositoryStatusError'
            }
          },
          entry: 'clearFirewallRepositoryStatusError'
        },
        enablingHealthCheckSingleRepo: {
          invoke: {
            src: 'enableHealthCheckSingleRepo',
            onDone: {
              target: 'readingHealthCheck',
              actions: 'clearEnablingHealthCheckRepoName'
            },
            onError: {
              target: 'loaded',
              actions: 'setEnableHealthCheckError'
            }
          },
          entry: ['setEnablingHealthCheckRepoName', 'clearEnableHealthCheckError']
        },
        enablingHealthCheckAllRepos: {
          invoke: {
            src: 'enableHealthCheckAllRepos',
            onDone: {
              target: 'readingHealthCheck'
            },
            onError: {
              target: 'loaded',
              actions: 'setEnableHealthCheckError'
            }
          },
          entry: ['clearEnablingHealthCheckRepoName', 'clearEnableHealthCheckError']
        }
      }
    })
}).withConfig({
  actions: {
    setData: assign((_, event) => {
      const data = event.data?.data;
      return {
        data: data,
        pristineData: data
      };
    }),

    filterData: assign({
      data: ({filter, pristineData}, _) =>
        pristineData.filter(({name}) => name.toLowerCase().indexOf(filter.toLowerCase()) !== -1)
    }),

    setHealthCheck: assign({
      healthCheck: (_, event) => indexBy(prop('repositoryName'), event.data)
    }),
    setEnableHealthCheckError: assign({
      enableHealthCheckError: (_, event) => event.data?.message
    }),
    clearEnableHealthCheckError: assign({
      enableHealthCheckError: () => null
    }),
    setReadHealthCheckError: assign({
      readHealthCheckError: (_, event) => event.data?.message
    }),
    clearReadHealthCheckError: assign({
      readHealthCheckError: () => null
    }),
    setEnablingHealthCheckRepoName: assign({
      enablingHealthCheckRepoName: (_, event) => event.repoName
    }),
    clearEnablingHealthCheckRepoName: assign({
      enablingHealthCheckRepoName: () => null
    }),

    setFirewallRepositoryStatus: assign({
      firewallStatus: (_, event) => indexBy(prop('repositoryName'), event.data)
    }),
    setFirewallRepositoryStatusError: assign({
      readFirewallRepositoryStatusError: (_, event) => event.data?.message
    }),
    clearFirewallRepositoryStatusError: assign({
      readFirewallRepositoryStatusError: () => null
    })
  },
  services: {
    fetchData: () => Axios.get(INTERNAL.REPOSITORIES_DETAILS),

    enableHealthCheckSingleRepo: async (_, event) => {
      const response = await ExtAPIUtils.extAPIRequest(
        EXT.HEALTH_CHECK.ACTION,
        EXT.HEALTH_CHECK.METHODS.UPDATE,
        {data: [true, event.repoName, true]}
      );
      ExtAPIUtils.checkForError(response);
      return ExtAPIUtils.extractResult(response);
    },

    enableHealthCheckAllRepos: async () => {
      const response = await ExtAPIUtils.extAPIRequest(
        EXT.HEALTH_CHECK.ACTION,
        EXT.HEALTH_CHECK.METHODS.ENABLE_ALL,
        {data: [true]}
      );
      ExtAPIUtils.checkForError(response);
      return ExtAPIUtils.extractResult(response);
    },

    readHealthCheck: async () => {
      const response = await ExtAPIUtils.extAPIRequest(
        EXT.HEALTH_CHECK.ACTION,
        EXT.HEALTH_CHECK.METHODS.READ
      );
      ExtAPIUtils.checkForError(response);
      return ExtAPIUtils.extractResult(response);
    },

    readFirewallRepositoryStatus: async () => {
      const response = await ExtAPIUtils.extAPIRequest(
        EXT.FIREWALL_REPOSITORY_STATUS.ACTION,
        EXT.FIREWALL_REPOSITORY_STATUS.METHODS.READ
      );
      ExtAPIUtils.checkForError(response);
      return ExtAPIUtils.extractResult(response);
    }
  },
  guards: {
    // Only request health check if user has permission AND NOT (IQ enabled AND Firewall enabled)
    // Health Check should be requested when: IQ is disabled OR IQ doesn't have Firewall
    shouldRequestHealthCheck: () => canUpdateHealthCheck() && !(isIqServerEnabled() && hasFirewall()),
    shouldRequestFirewallStatus: () => isIqServerEnabled() && canReadFirewallStatus()
  }
});
