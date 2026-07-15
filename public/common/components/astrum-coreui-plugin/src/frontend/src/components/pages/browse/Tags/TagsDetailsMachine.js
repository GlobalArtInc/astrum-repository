
import {assign, Machine} from 'xstate';
import Axios from "axios";
import {APIConstants} from '@globalart/astrum-ui-plugin';
import UIStrings from '../../../../constants/UIStrings';

const baseUrl = APIConstants.REST.PUBLIC.TAGS;
const urlWithTag = (tagName) => `${baseUrl}/${tagName}`
const {TAGS} = UIStrings;

export default Machine(
{
  id: 'TagsDetailsMachine',
  initial: 'loading',

  states: {
    loading: {
      invoke: {
        src: 'fetchData',
        onDone: {
          target: 'loaded',
          actions: ['setData', 'clearError']
        },
        onError: {
          target: 'loadError',
          actions: ['setError']
        },
      }
    },
    loaded: {},
    loadError: {
      on: {
        'RETRY': {
          target: 'loading',
        }
      }
    },
  },
},
{
  actions: {
    setData: assign({
      data: (_, {data}) => data.data,
      pristineData: (_, {data}) => data.data,
    }),
    setError: assign({
      error: (_, event) => {
        const tagNotFound = event.data?.message == "Request failed with status code 404";
        const message = tagNotFound ? TAGS.DETAILS.TAG_NOT_FOUND : event.data?.message;
        return message ? `${message}` : null;
      },
    }),
    clearError: assign({
      error: null
    }),
  },
  services: {
    fetchData: ({pristineData}) => Axios.get(urlWithTag(pristineData.name))
  }
});
