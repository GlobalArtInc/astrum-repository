
package org.globalart.astrum.common.upgrade.events;

import java.util.Collection;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * An event fired when a database migration has completed.
 */
public class UpgradeCompletedEvent
    extends UpgradeEventSupport
{
  private Collection<String> nodeIds;

  protected UpgradeCompletedEvent() {
    // deserialization
  }

  public UpgradeCompletedEvent(
      @Nullable final String user,
      final String schemaVersion,
      final Collection<String> nodeIds,
      final String... migrations)
  {
    super(user, schemaVersion, migrations);
    this.nodeIds = checkNotNull(nodeIds);
  }

  /**
   * Return the nodes who participated in the quorum
   */
  public Collection<String> getNodeIds() {
    return nodeIds;
  }

  public void setNodeIds(final Collection<String> nodeIds) {
    this.nodeIds = nodeIds;
  }
}
