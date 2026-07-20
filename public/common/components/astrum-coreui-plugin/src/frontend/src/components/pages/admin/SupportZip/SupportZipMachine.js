
import {assign, Machine} from 'xstate';
import Axios from "axios";

export default Machine(
    {
      initial: 'selectContents',

      context: {
        params: {
          systemInformation: true,
          threadDump: true,
          configuration: true,
          security: true,
          log: true,
          taskLog: true,
          auditLog: true,
          metrics: true,
          jmx: true,
          replication: true,
          archivedLog: 0,
          limitFileSizes: true,
          limitZipSize: true
        },
        response: {}
      },

      states: {
        selectContents: {
          on: {
            UPDATE: {
              target: 'selectContents',
              actions: ['setParams']
            },
            CREATE_SUPPORT_ZIPS: {
              target: 'creatingSupportZips'
            },
            CREATE_HA_SUPPORT_ZIPS: {
              target: 'creatingHaSupportZips'
            }
          }
        },
        creatingSupportZips: {
          invoke: {
            src: 'create',
            onDone: {
              target: 'supportZipsCreated',
              actions: ['setResponse']
            },
            onError: {
              target: 'error',
              actions: ['setCreateError']
            }
          }
        },
        supportZipsCreated: {},
        creatingHaSupportZips: {
          invoke: {
            src: 'createHaZips',
            onDone: {
              target: 'haSupportZipsCreated',
              actions: ['setResponse']
            },
            onError: {
              target: 'error',
              actions: ['setCreateError']
            }
          }
        },
        haSupportZipsCreated: {},
        error: {
          on: {
            RETRY: {
              target: 'creatingSupportZips'
            },
            RETRY_HA: {
              target: 'creatingHaSupportZips'
            }
          }
        }
      },
    },
    {
      actions: {
        setParams: assign({
          params: (_, {params}) => params
        }),
        setCreateError: assign({
          createError: (_, event) => event.data.message
        }),
        setResponse: assign({
          response: (_, event) => event.data.data,
        })
      },
      services: {
        create: ({params}) => Axios.post('service/rest/v1/support/supportzippath', params),
        createHaZips: ({params}) => Axios.post('service/rest/v1/nodes/supportzips', params)
      }
    }
);
