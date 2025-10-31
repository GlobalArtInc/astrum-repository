
package org.globalart.astrum.common.upgrade.events;

import javax.annotation.Nullable;

/**
 * An event which is fired when a database migration is starting
 */
public class UpgradeStartedEvent
    extends UpgradeEventSupport
{
  protected UpgradeStartedEvent() {
    // deserialization
  }

  public UpgradeStartedEvent(@Nullable final String user, final String schemaVersion, final String... migrations) {
    super(user, schemaVersion, migrations);
  }
}
