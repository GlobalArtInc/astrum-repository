
package org.globalart.astrum.repository.config.internal;

import org.globalart.astrum.common.event.EventWithSource;
import org.globalart.astrum.repository.config.Configuration;

import com.fasterxml.jackson.annotation.JsonIgnore;

import static com.google.common.base.Preconditions.checkNotNull;

public class ConfigurationDeletedEvent
    extends EventWithSource
    implements org.globalart.astrum.repository.config.ConfigurationDeletedEvent
{
  private String repositoryName;

  public ConfigurationDeletedEvent() {
    // deserialization
  }

  public ConfigurationDeletedEvent(final ConfigurationData configuration) {
    this.repositoryName = checkNotNull(configuration).getRepositoryName();
  }

  @Override
  public String getRepositoryName() {
    return repositoryName;
  }

  public void setRepositoryName(final String repositoryName) {
    this.repositoryName = repositoryName;
  }

  @JsonIgnore
  @Override
  public Configuration getConfiguration() {
    throw new UnsupportedOperationException("Configuration is not available in the event, use DB instead");
  }
}
