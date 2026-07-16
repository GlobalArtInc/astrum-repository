
import {assign, createMachine} from 'xstate';
import Axios from 'axios';

export const ASC = 1;
export const DESC = -1;

export default createMachine(
    {
      id: 'ContentSelectorsPreview',

      initial: 'loading',

      context: {
        type: '',
        expression: '',
        allRepositories: [],
        previewResponse: [],
        preview: [],
        previewError: '',
        repositories: '*',
        filterText: '',
        error: ''
      },

      states: {
        loading: {
          invoke: {
            id: 'fetchRepositories',
            src: 'fetchRepositories',
            onDone: {
              target: 'loaded',
              actions: ['setAllRepositories']
            },
            onError: {
              target: 'loadError',
              actions: ['setLoadError', 'logLoadError']
            }
          }
        },
        loaded: {
          entry: ['filterPreview'],
          on: {
            SET_REPOSITORIES: {
              target: 'loaded',
              actions: ['setRepositories']
            },
            PREVIEW: {
              target: 'preview',
              actions: ['clearPreviewError', 'clearFilter']
            },
            FILTER: {
              target: 'loaded',
              actions: ['setFilter']
            }
          }
        },
        preview: {
          invoke: {
            id: 'preview',
            src: 'preview',
            onDone: {
              target: 'loaded',
              actions: ['setPreview']
            },
            onError: {
              target: 'loaded',
              actions: ['setPreviewError']
            }
          }
        },
        loadError: {}
      },
      on: {
        RETRY: {
          target: 'loading'
        }
      }
    },
    {
      actions: {
        setAllRepositories: assign({
          allRepositories: (_, event) => event.data.data
        }),

        setRepositories: assign({
          repositories: (_, {repositories}) => repositories
        }),

        clearFilter: assign({
          filterText: () => ''
        }),

        setFilter: assign({
          filterText: (_, {filter}) => filter
        }),

        filterPreview: assign({
          preview: ({filterText, previewResponse}, _) => previewResponse.filter(name =>
              name.toLowerCase().indexOf(filterText.toLowerCase()) !== -1
          )
        }),

        setLoadError: assign({
          error: (_, event) => event.data.message
        }),

        setPreview: assign({
          previewResponse: (_, event) => event.data.data.results.map(({name}) => name)
        }),

        setPreviewError: assign({
          previewError: (_, event) => event.data.response.data[0].message
        }),

        clearPreviewError: assign({
          previewError: () => ''
        }),

        logLoadError: (_, event) => console.error(event.data.message)
      },
      services: {
        fetchRepositories: () => Axios.get('service/rest/internal/ui/repositories?withAll=true&withFormats=true'),
        preview: ({repositories}, {selectorType, expression}) => Axios.post('service/rest/internal/ui/content-selectors/preview', {
          repository: repositories,
          type: selectorType,
          expression
        })
      }
    }
);
