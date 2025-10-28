
package org.globalart.astrum.common.node;

import java.io.Serializable;
import java.util.Objects;

/**
 * Node configuration
 *
 * @since 3.6
 */
public class NodeConfiguration
    implements Serializable
{

  private static final long serialVersionUID = 5687759567911666915L;

  /**
   * UUID identifying cluster node
   */
  private String id;

  /**
   * Admin-specified node identifier
   */
  private String friendlyNodeName;

  public NodeConfiguration() {
    // default constructor
  }

  public NodeConfiguration(final String id, final String friendlyNodeName) {
    this.id = id;
    this.friendlyNodeName = friendlyNodeName;
  }

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public String getFriendlyNodeName() {
    return friendlyNodeName;
  }

  public void setFriendlyNodeName(final String friendlyNodeName) {
    this.friendlyNodeName = friendlyNodeName;
  }

  @Override
  public boolean equals(final Object obj) {
    if (obj == null) {
      return false;
    }
    if (getClass() != obj.getClass()) {
      return false;
    }
    final NodeConfiguration other = (NodeConfiguration) obj;
    return Objects.equals(this.getId(), other.getId())
        && Objects.equals(this.getFriendlyNodeName(), other.getFriendlyNodeName());
  }

  @Override
  public int hashCode() {
    return Objects.hash(getId(), getFriendlyNodeName());
  }
}
