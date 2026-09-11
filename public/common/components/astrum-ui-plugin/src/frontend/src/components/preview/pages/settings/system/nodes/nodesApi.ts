

import { ExtAPIUtils } from '../../../../../../interface/ExtAPIUtils';
import { APIConstants } from '../../../../../../constants/APIConstants';
import { restClient } from '../../../../../../interface/api';
import { ExtJS } from '../../../../../../interface/ExtJS';
import { NodeInfo } from './types';

const { REST } = APIConstants;
const ACTIVE_NODES_URL = REST.INTERNAL.GET_SUPPORT_ZIP_ACTIVE_NODES;
const NODE_ACTION = 'node_NodeAccess';
const CLUSTERED_ENABLED_KEY = 'astrum.datastore.clustered.enabled';

interface ActiveNodeData {
  nodeId: string;
  hostname: string;
}

/**
 * Fetch cluster nodes, merging the ExtDirect node list (name + local flag)
 * with best-effort hostname resolution from the REST active-nodes endpoint.
 * Hostname resolution failures fall back to the node name.
 */
export async function fetchNodes(): Promise<NodeInfo[]> {
  const response = await ExtAPIUtils.extAPIRequest(NODE_ACTION, 'nodes');
  ExtAPIUtils.checkForError(response);
  const extNodes: Array<{ name: string; local: boolean }> =
    Array.isArray(response?.data?.result?.data) ? response.data.result.data : [];

  let hostnameByNodeId = new Map<string, string>();
  // The activenodes endpoint is only registered in clustered (HA) mode (backend
  // @ConditionalOnProperty astrum.datastore.clustered.enabled), so it returns 404
  // in single-node mode. Only attempt the hostname lookup when clustering is
  // enabled; otherwise nodes fall back to their id/name (the same result the 404
  // fallback produced, but without the pointless failed request).
  if (ExtJS.state().getValue(CLUSTERED_ENABLED_KEY)) {
    try {
      const activeNodes = await restClient.get<ActiveNodeData[]>(ACTIVE_NODES_URL);
      hostnameByNodeId = new Map((activeNodes ?? []).map((n) => [n.nodeId, n.hostname]));
    } catch (_err) {
      // Best-effort; fall back to node name below.
    }
  }

  return extNodes.map((node) => ({
    name: node.name,
    displayName: hostnameByNodeId.get(node.name) || node.name,
    local: node.local,
  }));
}
