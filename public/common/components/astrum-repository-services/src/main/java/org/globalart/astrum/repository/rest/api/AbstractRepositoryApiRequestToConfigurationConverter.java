
package org.globalart.astrum.repository.rest.api;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.repository.rest.api.model.AbstractRepositoryApiRequest;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.DATA_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;

/**
 * @since 3.20
 */
public abstract class AbstractRepositoryApiRequestToConfigurationConverter<T extends AbstractRepositoryApiRequest>
{
  protected ConfigurationStore configurationStore;

  @Autowired
  public void setConfigurationStore(final ConfigurationStore configurationStore) {
    this.configurationStore = checkNotNull(configurationStore);
  }

  protected void maybeAddDataStoreName(final Configuration configuration) {
    configuration.attributes(STORAGE).set(DATA_STORE_NAME, DEFAULT_DATASTORE_NAME);
  }

  public Configuration convert(final T request) {
    Configuration configuration = configurationStore.newConfiguration();
    configuration.setRepositoryName(request.getName());
    configuration.setRecipeName(String.join("-", request.getFormat(), request.getType()));
    configuration.setOnline(request.getOnline());
    return configuration;
  }
}
