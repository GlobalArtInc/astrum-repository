
package org.globalart.astrum.security.realm;

/**
 * Emitted when {@link RealmConfiguration} has been changed.
 *
 * @since 3.1
 */
public class RealmConfigurationChangedEvent
{
  private final RealmConfiguration configuration;

  public RealmConfigurationChangedEvent(final RealmConfiguration configuration) {
    this.configuration = configuration;
  }

  public RealmConfiguration getConfiguration() {
    return configuration;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "configuration=" + configuration +
        '}';
  }
}
