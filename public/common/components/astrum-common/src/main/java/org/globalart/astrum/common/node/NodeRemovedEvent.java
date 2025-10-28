
package org.globalart.astrum.common.node;

/**
 * Emitted when a node has been removed from the cluster.
 *
 * @since 3.1
 */
public class NodeRemovedEvent
    extends NodeEvent
{
  public NodeRemovedEvent(final String nodeId) {
    super(nodeId);
  }
}
