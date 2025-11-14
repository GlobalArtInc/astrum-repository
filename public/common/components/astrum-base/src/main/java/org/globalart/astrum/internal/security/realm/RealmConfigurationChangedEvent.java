
package org.globalart.astrum.internal.security.realm;

import org.globalart.astrum.common.event.EventWithSource;
import org.globalart.astrum.security.realm.RealmConfiguration;
import org.globalart.astrum.security.realm.RealmConfigurationEvent;

public class RealmConfigurationChangedEvent
    extends EventWithSource
    implements RealmConfigurationEvent
{
  private RealmConfigurationData configuration;

  public RealmConfigurationChangedEvent(final RealmConfigurationData configuration) {
    this.configuration = configuration;
  }

  RealmConfigurationChangedEvent() {
    // deserialization
  }

  public void setConfiguration(final RealmConfigurationData configuration) {
    this.configuration = configuration;
  }

  @Override
  public RealmConfiguration getConfiguration() {
    return configuration;
  }
}
