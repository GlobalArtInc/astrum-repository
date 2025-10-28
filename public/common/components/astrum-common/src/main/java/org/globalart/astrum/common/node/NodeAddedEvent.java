
package org.globalart.astrum.common.node;

/**
 * Emitted when a node has been added to the cluster.
 *
 * @since 3.1
 */
public class NodeAddedEvent
    extends NodeEvent
{
  public NodeAddedEvent(final String nodeId) {
    super(nodeId);
  }
}
