
package org.globalart.astrum.webhooks;

import java.util.Date;

import org.globalart.astrum.common.node.NodeAccess;

/**
 * The base structure for all webhook payloads.
 *
 * @since 3.1
 */
public abstract class WebhookPayload
{
  /**
   * When the change occurred.
   */
  private Date timestamp;

  /**
   * The node-id where the change occurred.
   *
   * @see NodeAccess#getId()
   */
  private String nodeId;

  /**
   * Who initiated the change.
   *
   * ex: 'admin/192.168.0.57'
   */
  private String initiator;

  public Date getTimestamp() {
    return timestamp;
  }

  public void setTimestamp(final Date timestamp) {
    this.timestamp = timestamp;
  }

  public String getNodeId() {
    return nodeId;
  }

  public void setNodeId(final String nodeId) {
    this.nodeId = nodeId;
  }

  public String getInitiator() {
    return initiator;
  }

  public void setInitiator(final String initiator) {
    this.initiator = initiator;
  }
}
