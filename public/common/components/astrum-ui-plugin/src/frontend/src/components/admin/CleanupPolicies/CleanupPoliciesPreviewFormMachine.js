
import {assign, createMachine} from 'xstate';
import Axios from 'axios';

export default createMachine({
  id: 'CleanupPoliciesPreviewFormMachine',
  initial: 'loaded',

  context: {
    repositories: [],
    repository: '',
    error: null
  },

  states: {
    loaded: {
      on: {
        LOAD_REPOSITORIES: [{
          target: 'loading',
          cond: 'canLoadRepositories'
        }, {
          actions: ['clearRepositories']
        }],
        SET_REPOSITORY: {
          actions: ['setRepository']
        },
        SET_FORMAT: {
          target: 'loading',
          actions: 'setFormat'
        }
      }
    },
    loading: {
      invoke: {
        src: 'fetchRepositories',
        onDone: [{
          target: 'loaded',
          actions: 'setRepositories'
        }],
        onError: {
          target: 'error',
          actions: ['setError']
        }
      }
    },
    error: {
      on: {
        SET_FORMAT: {
          target: 'loading',
          actions: ['clearError']
        }
      }
    }
  }
}, {
  actions: {
    clearRepositories: assign({
      repository: () => '',
      repositories: () => []
    }),

    clearError: assign({
      error: () => null
    }),

    setRepository: assign({
      repository: (_, {repository}) => repository
    }),

    setRepositories: assign({
      repositories: (_, event) => event.data?.data
    }),

    setError: assign({
      error: (_, event) => event.data.message
    })
  },
  guards: {
    canLoadRepositories: (_, {format}) => Boolean(format)
  },
  services: {
    fetchRepositories: (_, {format}) => Axios.get('service/rest/internal/ui/repositories', {params: {format}})
  }
});
