
package org.globalart.astrum.internal.security.anonymous;

import org.globalart.astrum.common.event.EventWithSource;
import org.globalart.astrum.security.anonymous.AnonymousConfiguration;

public class AnonymousConfigurationUpdatedEvent
    extends EventWithSource
    implements AnonymousConfigurationEvent
{
  private AnonymousConfigurationData anonymousConfiguration;

  public AnonymousConfigurationUpdatedEvent() {
    // deserialization
  }

  public AnonymousConfigurationUpdatedEvent(final AnonymousConfigurationData anonymousConfiguration) {
    this.anonymousConfiguration = anonymousConfiguration;
  }

  @Override
  public AnonymousConfiguration getAnonymousConfiguration() {
    return anonymousConfiguration;
  }

  public void setAnonymousConfiguration(final AnonymousConfigurationData anonymousConfiguration) {
    this.anonymousConfiguration = anonymousConfiguration;
  }
}
