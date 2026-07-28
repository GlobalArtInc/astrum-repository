
import {assign, createMachine} from 'xstate';
import Axios from 'axios';

const INITIAL_DATA = {
  loadError: null,
  repositories: [],
  repository: ''
};

const cleanupPoliciesDryRunMachine = createMachine({
  id: 'CleanupPoliciesDryRunMachine',
  initial: 'loaded',

  context: INITIAL_DATA,

  states: {
    loaded: {
      on: {
        SET_REPOSITORY: {
          actions: 'setRepository'
        }
      }
    },
    loading: {
      entry: 'resetData',

      invoke: {
        src: 'fetchRepositories',
        onDone: [{
          target: 'loaded',
          actions: 'setRepositories'
        }],
        onError: {
          target: 'error',
          actions: 'setError'
        }
      }
    },
    error: {
      on: {
        RETRY: {
          target: 'loading',
          actions: 'clearError'
        }
      }
    }
  },

  on: {
    LOAD_REPOSITORIES: {
      target: 'loading',
      cond: 'canLoadRepositories'
    }
  }
}, {
  actions: {
    resetData: assign({
      loadError: INITIAL_DATA.loadError,
      repositories: INITIAL_DATA.repositories,
      repository: INITIAL_DATA.repository
    }),

    setRepositories: assign({
      repositories: (_, event) => event.data?.data
    }),

    setRepository: assign({
      repository: (_, {repository}) => repository
    }),

    setError: assign({
      loadError: (_, event) => event.data.message
    }),

    clearError: assign({
      loadError: () => null
    })
  },
  guards: {
    canLoadRepositories: (_, {format}) => Boolean(format)
  },
  services: {
    fetchRepositories: (_, {format}) => Axios.get('service/rest/internal/ui/repositories', {params: {format}})
  }
});

export default cleanupPoliciesDryRunMachine;
