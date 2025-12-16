

package org.globalart.astrum.security.anonymous;

/**
 * Emitted when {@link AnonymousConfiguration} has changed.
 *
 * @since 3.1
 */
public class AnonymousConfigurationChangedEvent
{
  private final AnonymousConfiguration configuration;

  public AnonymousConfigurationChangedEvent(final AnonymousConfiguration configuration) {
    this.configuration = configuration;
  }

  public AnonymousConfiguration getConfiguration() {
    return configuration;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "configuration=" + configuration +
        '}';
  }
}
