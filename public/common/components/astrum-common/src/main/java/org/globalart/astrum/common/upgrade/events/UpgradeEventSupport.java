
package org.globalart.astrum.common.upgrade.events;

import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.event.EventWithSource;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Support class for the database migration events
 */
public abstract class UpgradeEventSupport
    extends EventWithSource
{
  private String user;

  private String schemaVersion;

  private String[] migrations;

  protected UpgradeEventSupport() {
    // deserialization
  }

  protected UpgradeEventSupport(
      @Nullable final String user,
      @Nullable final String schemaVersion,
      final String[] migrations)
  {
    this.user = user;
    this.schemaVersion = schemaVersion;
    this.migrations = checkNotNull(migrations);
  }

  /**
   * @return the migrations associated with the event
   */
  public String[] getMigrations() {
    return migrations;
  }

  /**
   * @return the database schema version at the event time, may be null if this is the first system start
   */
  public Optional<String> getSchemaVersion() {
    return Optional.ofNullable(schemaVersion);
  }

  /**
   * @return the userId of the person who triggered it, or empty if this was triggered automatically
   */
  public Optional<String> getUser() {
    return Optional.ofNullable(user);
  }

  public void setMigrations(final String[] migrations) {
    this.migrations = migrations;
  }

  public void setSchemaVersion(final String schemaVersion) {
    this.schemaVersion = schemaVersion;
  }

  public void setUser(final String user) {
    this.user = user;
  }
}
