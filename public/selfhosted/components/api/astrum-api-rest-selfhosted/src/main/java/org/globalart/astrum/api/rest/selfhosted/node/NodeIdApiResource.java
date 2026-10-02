
package org.globalart.astrum.api.rest.selfhosted.node;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Consumes;
import jakarta.ws.rs.DELETE;
import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.node.datastore.NodeIdStore;
import org.globalart.astrum.rest.Resource;

import org.apache.shiro.authz.annotation.RequiresAuthentication;
import org.apache.shiro.authz.annotation.RequiresPermissions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * REST API to reset the stored Node ID. This is intended for use when cloning a system.
 *
 * @since 3.37
 */
@Consumes(APPLICATION_JSON)
@Produces(APPLICATION_JSON)
@Path(NodeIdApiResource.PATH)
@Component
public class NodeIdApiResource
    implements Resource, NodeIdApiResourceDoc
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String PATH = V1_API_PREFIX + "/system/node";

  private final NodeIdStore nodeIdStore;

  private final NodeAccess nodeAccess;

  @Autowired
  public NodeIdApiResource(final NodeIdStore nodeIdStore, final NodeAccess nodeAccess) {
    this.nodeIdStore = nodeIdStore;
    this.nodeAccess = nodeAccess;
  }

  @Override
  @GET
  @RequiresAuthentication
  @RequiresPermissions("astrum:*")
  public NodeInformation getNodeId() {
    // we return an object here to maintain an API similar to multi-node
    return new NodeInformation(nodeAccess.getId());
  }

  @Override
  @DELETE
  @RequiresAuthentication
  @RequiresPermissions("astrum:*")
  public void clear() {
    nodeIdStore.clear();
  }

  public static class NodeInformation
  {
    public final String nodeId;

    public NodeInformation(final String nodeId) {
      this.nodeId = nodeId;
    }
  }
}
