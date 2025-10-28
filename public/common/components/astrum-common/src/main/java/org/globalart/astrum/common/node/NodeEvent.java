
package org.globalart.astrum.common.node;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Abstract super class for node events.
 *
 * @since 3.1
 */
public abstract class NodeEvent
{
  private final String nodeId;

  public NodeEvent(final String nodeId) {
    this.nodeId = checkNotNull(nodeId);
  }

  public String getNodeId() {
    return nodeId;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "nodeId='" + nodeId + '\'' +
        '}';
  }
}
