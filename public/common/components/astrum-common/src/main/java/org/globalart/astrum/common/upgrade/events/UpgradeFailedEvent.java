
package org.globalart.astrum.common.upgrade.events;

import javax.annotation.Nullable;

/**
 * An event fired when a database migration fails.
 */
public class UpgradeFailedEvent
    extends UpgradeEventSupport
{
  private String errorMessage;

  protected UpgradeFailedEvent() {
    // deserialization
  }

  public UpgradeFailedEvent(
      @Nullable final String user,
      final String schemaVersion,
      final String errorMessage,
      final String... migrations)
  {
    super(user, schemaVersion, migrations);
    this.errorMessage = errorMessage;
  }

  /**
   * @return the error message associated with the failure
   */
  public String getErrorMessage() {
    return errorMessage;
  }

  public void setErrorMessage(final String errorMessage) {
    this.errorMessage = errorMessage;
  }
}
