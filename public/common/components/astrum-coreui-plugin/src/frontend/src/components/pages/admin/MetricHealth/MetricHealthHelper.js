
import {ExtJS} from '@globalart/astrum-ui-plugin';
const baseUrl = 'service/rest/beta/status/check';
const nodesUrl = `${baseUrl}/cluster`;
const singleNodeUrl = (nodeId) => `${baseUrl}/${nodeId}`;

export const URL = {
  nodesUrl,
  singleNodeUrl,
};

export const isClustered = () =>
  ExtJS.state().getValue('astrum.datastore.clustered.enabled');
