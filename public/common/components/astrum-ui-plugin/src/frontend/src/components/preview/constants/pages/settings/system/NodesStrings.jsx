import React from 'react';
import localize from '../../../../../../interface/localize';

export default localize('react-ui', {
  LIST: {
    LOADING_NODES: 'Loading nodes...',
    NODE_NAME: 'Node Name',
    NODE_IDENTITY: 'Node Identity',
    NO_NODES_FOUND: 'No nodes found',
    NODE_YOU_CURRENTLY_CONNECTED: 'The node that you are currently connected to',
    CURRENT_NODE: 'Current Node',
  },
  MACHINE: {
    FAILED_LOAD_NODES: 'Failed to load nodes',
  },
  PAGE: {
    SINGLE_NODE_MODE: (link) => <>This instance is running in single-node mode. High Availability cluster nodes are not expected. See the {link} for more information.</>,
    HIGH_AVAILABILITY_DOCUMENTATION: 'high availability documentation',
    NODES: 'Nodes',
    VIEW_CLUSTER_NODES_ASTRUM_REPOSITORY: 'View cluster nodes in this Astrum Repository instance',
    NODES_PARTICIPATING_ASTRUM_REPOSITORY_INSTANCE: 'Nodes participating in this Astrum Repository instance.',
  },
}, 'PREVIEW.NODES');
