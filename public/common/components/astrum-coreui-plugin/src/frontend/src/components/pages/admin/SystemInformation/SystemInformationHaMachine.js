
import {assign, Machine} from 'xstate';
import Axios from 'axios';
import {APIConstants} from '@globalart/astrum-ui-plugin';

const {SYSTEM_INFORMATION_HA, SYSTEM_INFORMATION} = APIConstants.REST;

export default Machine(
  {
    id: 'SystemInformation',
    initial: 'loading',

    context: {
      selectedNodeId: '',
      nodesMapById: {},
      systemInformation: {},
      nodeIds: []
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
      loaded: {
        on: {
          SELECT_NODE: {
            actions: 'setSelectedNodeId'
          }
        }
      },
      loadError: {
        on: {
          RETRY: {
            target: 'loading'
          }
        }
      }
    }
  },
  {
    actions: {
      setData: assign((ctx, event) => {
        const atlasSystemInformation = event.data[0]?.data;
        const clusterInformation = event.data[1]?.data;

        const selectedNodeId = atlasSystemInformation['astrum-node']['node-id'];
        const nodesMapById = getNodesMapById(clusterInformation);

        return {
          ...ctx,
          selectedNodeId,
          nodesMapById,
          systemInformation: nodesMapById[selectedNodeId],
          nodeIds: Object.keys(nodesMapById)
        };
      }),
      setSelectedNodeId: assign({
        selectedNodeId: (_, {value}) => value,
        systemInformation: ({nodesMapById}, {value}) => nodesMapById[value]
      }),
      logError: (_, event) => console.error('Failed to load System Information', event)
    },
    services: {
      fetchData: () => Axios.all([Axios.get(SYSTEM_INFORMATION), Axios.get(SYSTEM_INFORMATION_HA)])
    }
  }
);

const getNodesMapById = (clusterInformation) =>
  Object.entries(clusterInformation).reduce((map, entry) => {
    const [sectionName, section] = entry;
    Object.entries(section).forEach((entry) => {
      const [nodeId, nodeData] = entry;
      map[nodeId] = map[nodeId] || {};
      map[nodeId][sectionName] = nodeData;
    });
    return map;
  }, {});
