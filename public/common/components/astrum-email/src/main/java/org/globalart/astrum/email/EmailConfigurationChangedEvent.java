

package org.globalart.astrum.email;

/**
 * Emitted when {@link EmailConfiguration} has been changed.
 *
 * @since 3.1
 */
public class EmailConfigurationChangedEvent
{
  private final EmailConfiguration configuration;

  public EmailConfigurationChangedEvent(final EmailConfiguration configuration) {
    this.configuration = configuration;
  }

  public EmailConfiguration getConfiguration() {
    return configuration;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "configuration=" + configuration +
        '}';
  }
}
