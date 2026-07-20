
import {assign, Machine} from 'xstate';
import Axios from "axios";
import {APIConstants} from '@globalart/astrum-ui-plugin';

const {SYSTEM_INFORMATION} = APIConstants.REST;

export default Machine(
{
  id: 'SystemInformation',
  initial: 'loading',

  context: {
    systemInformation: {}
  },

  states: {
    loading: {
      invoke: {
        src: 'fetchData',
        onDone: {
          target: 'loaded',
          actions: ['setData']
        },
        onError: {
          target: 'loadError',
          actions: ['logError']
        }
      }
    },
    loaded: {},
    loadError: {  
      on: {
        'RETRY': {
          target: 'loading'
        }
      }
    }
  },

},
{
  actions: {
    setData: assign({
      systemInformation: (_, event) => event.data?.data
    }),
    logError: (_, event) => console.error('Failed to load System Information', event)
  },
  services: {
    fetchData: () => Axios.get(SYSTEM_INFORMATION)
  }
});
