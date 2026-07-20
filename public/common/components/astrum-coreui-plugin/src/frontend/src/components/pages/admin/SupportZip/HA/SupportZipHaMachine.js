
import Axios from 'axios';
import {mergeDeepRight} from 'ramda';
import {assign, spawn, actions} from 'xstate';
import {APIConstants, FormUtils} from '@globalart/astrum-ui-plugin';
import {cleanNode} from './NodeCardHelper';
import NodeCardMachine from './NodeCardMachine';

const getNode = async (nodeId, params) => {
  return await Axios.post(
    APIConstants.REST.INTERNAL.SUPPORT_ZIP + nodeId,
    params
  );
};

const generateForNode = async (nodeId, params, ctx) => {
  const node = ctx.astrumNodes.find(node => node.nodeId === nodeId);
  node.machineRef.send({
    type: 'UPDATE_STATUS',
    status: 'CREATING',
  });

  return getNode(nodeId, params);
};

const sharedEvents = {
  CREATE_SUPPORT_ZIP_FOR_NODE: {
    target: 'createSingleNodeSupportZip',
    actions: 'setNode',
  },
  CREATE_SUPPORT_ZIP_FOR_ALL_NODES: {
    target: 'createAllNodesSupportZip',
    actions: 'clearLoadError',
  },
};

export default FormUtils.buildFormMachine({
  id: 'SupportZipHaMachine',
  config: (config) =>
    mergeDeepRight(config, {
      context: {
        astrumNodes: [],
        selectedNode: null,
        showCreateZipModal: false,
        targetNode: null,
        isBlobStoreConfigured: false,
      },
      states: {
        loaded: {
          id: 'loaded',
          initial: 'idle',
          states: {
            idle: {
              on: {
                ...sharedEvents,
                REMOVE_ACTORS: {
                  actions: 'removeActors',
                },
              },
            },
            createAllNodesSupportZip: {
              on: {
                GENERATE: {
                  target: 'creatingAllNodesSupportZip',
                  actions: 'updateAllNodeStatus',
                },
                CANCEL: {
                  target: '#loaded',
                },
              },
            },
            createSingleNodeSupportZip: {
              on: {
                GENERATE: {
                  target: 'creatingSingleNodeSupportZip',
                  actions: 'updateNodeStatus',
                },
                CANCEL: {
                  target: '#loaded',
                  actions: 'setNode',
                },
              },
            },
            creatingAllNodesSupportZip: {
              on: sharedEvents,
              invoke: {
                src: 'createAllNodeSupportZip',
                onDone: {
                  target: '#loaded',
                  actions: 'updateAllNodesAfterGenerate',
                },
                onError: {
                  target: '#loaded',
                  actions: 'setLoadError',
                },
              },
            },
            creatingSingleNodeSupportZip: {
              on: sharedEvents,
              invoke: {
                src: 'createSupportZip',
                onDone: {
                  target: '#loaded',
                  actions: 'updateNodeAfterGenerate',
                },
                onError: '#loaded',
              },
            },
          },
        },
      },
    }),
}).withConfig({
  actions: {
    setData: assign((context, event) => {
      const [nodes, blobStores] = event.data;
      const isBlobStoreConfigured = blobStores.data.length > 0;
      const astrumNodes = nodes.data.map((node) => {
        const id = `node-${node.nodeId}`;
        // Removes any previous instance with the same id
        actions.stop(id);

        return mergeDeepRight(node, {
          machineRef: spawn(
            NodeCardMachine.withContext({data: node, pristineData: node}),
            id
          ),
        });
      });

      return mergeDeepRight(context, {
        astrumNodes: astrumNodes,
        isBlobStoreConfigured,
      });
    }),
    setNode: assign({
      selectedNode: (_, event) => event.node ?? null,
    }),
    updateNodeStatus: assign({
      selectedNode: (context) => {
        // Update local status optimistically without triggering a fetch
        // The fetch will happen after POST completes in updateNodeAfterGenerate
        context.selectedNode.machineRef.send({
          type: 'SET_STATUS_LOCAL',
          status: 'CREATING',
        });
        return context.selectedNode;
      },
    }),
    updateAllNodeStatus: assign({
      astrumNodes: (context) => {
        // Update local status optimistically without triggering fetches
        // The fetches will happen after POST completes in updateAllNodesAfterGenerate
        context.astrumNodes.forEach((node) => {
          node.machineRef.send({
            type: 'SET_STATUS_LOCAL',
            status: 'CREATING',
          });
        });
        return context.astrumNodes;
      },
    }),
    updateNodeAfterGenerate: (context) => {
      // Trigger the selected node's machine to refresh its status from the server
      if (context.selectedNode && context.selectedNode.machineRef) {
        context.selectedNode.machineRef.send({
          type: 'UPDATE_STATUS',
          status: 'CREATING',
        });
      }
    },
    updateAllNodesAfterGenerate: (context) => {
      // Trigger all node machines to refresh their status from the server
      context.astrumNodes.forEach((node) => {
        node.machineRef.send({
          type: 'UPDATE_STATUS',
          status: 'CREATING',
        });
      });
    },
  },
  services: {
    fetchData: () => {
      return Axios.all([
        Axios.get(APIConstants.REST.INTERNAL.GET_SUPPORT_ZIP_ACTIVE_NODES),
        Axios.get(APIConstants.REST.PUBLIC.BLOB_STORES),
      ]);
    },
    createAllNodeSupportZip: async (ctx, event) => {
      const zipParams = event.params || null;

      const deletePromises = ctx.astrumNodes.map((node) => {
        if (ctx.isBlobStoreConfigured) {
          return cleanNode(node.nodeId);
        }

        return Promise.resolve();
      });

      await Axios.all(deletePromises);

      const getPromises = ctx.astrumNodes.map((node) => {
        const params = {
          ...zipParams,
          hostname: node.hostname,
        };

        if (ctx.isBlobStoreConfigured) {
          return generateForNode(node.nodeId, params, ctx);
        }

        return Promise.resolve();
      });

      return Axios.all(getPromises);
    },
    createSupportZip: async (ctx, event) => {
      const node = ctx.selectedNode;
      const zipParams = event.params;

      if (node && zipParams && ctx.isBlobStoreConfigured) {
        const selectedNodeId = node.nodeId;
        const params = {
          ...zipParams,
          hostname: node.hostname,
        };

        await cleanNode(selectedNodeId);

        return getNode(selectedNodeId, params);
      }

      return Promise.reject();
    },
  },
});
