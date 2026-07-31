
import {assign, Machine} from 'xstate';
import Axios from 'axios';
import { APIConstants } from '../constants/APIConstants';

const getPublicAPI = () => APIConstants?.REST?.PUBLIC || {};

export default Machine(
  {
    id: 'GlobalEvaluationSettings',
    initial: 'loading',

    context: {
      data: null
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
            target: 'loaded',
            actions: ['setDataNull']
          }
        }
      },
      loaded: {}
    }
  },
  {
    actions: {
      setData: assign({
        data: (_, event) => {
          // Handle 204 No Content or empty response
          if (!event.data?.data) {
            return null;
          }
          return event.data.data;
        }
      }),
      setDataNull: assign({
        data: null
      })
    },
    services: {
      fetchData: () => {
        const PUBLIC = getPublicAPI();
        return Axios.get(PUBLIC.EVALUATION_SETTINGS);
      }
    }
  }
);
