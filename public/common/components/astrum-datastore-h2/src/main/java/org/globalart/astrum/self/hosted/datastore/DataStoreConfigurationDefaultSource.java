
package org.globalart.astrum.self.hosted.datastore;

import org.globalart.astrum.datastore.DataStoreConfigurationSource;
import org.globalart.astrum.datastore.api.DataStoreConfiguration;

import com.google.common.collect.ImmutableMap;
import com.google.common.collect.ImmutableSet;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkArgument;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.self.hosted.datastore.DataStoreConfigurationDefaultSource.DEFAULT;

/**
 * Source of {@link DataStoreConfiguration}s for the default datastore only
 *
 * @since 3.19
 */
@Component
@Qualifier(DEFAULT)
@Order(DataStoreConfigurationSource.ORDER_DEFAULT_SOURCE)
public class DataStoreConfigurationDefaultSource
    implements DataStoreConfigurationSource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  static final String DEFAULT = "default";

  private static final String JDBC_TEMPLATE_URL =
      "jdbc:h2:file:${karaf.data}/db/" + DEFAULT_DATASTORE_NAME + ";DB_CLOSE_ON_EXIT=FALSE";

  private static final String JDBC = "jdbc";

  private static final String JDBC_URL = "jdbcUrl";

  @Override
  public String getName() {
    return "Local";
  }

  @Override
  public boolean isModifiable() {
    return true;
  }

  @Override
  public Iterable<String> browseStoreNames() {
    return ImmutableSet.<String>builder().add(DEFAULT_DATASTORE_NAME).build();
  }

  @Override
  public DataStoreConfiguration load(final String storeName) {
    checkArgument(DEFAULT_DATASTORE_NAME.equalsIgnoreCase(storeName),
        "%s is not valid, %s is the only valid data store name", storeName, DEFAULT_DATASTORE_NAME);
    DataStoreConfiguration configuration = new DataStoreConfiguration();
    configuration.setName(DEFAULT_DATASTORE_NAME);
    configuration.setType(JDBC);
    configuration.setSource(DEFAULT);
    configuration.setAttributes(ImmutableMap.of(JDBC_URL, JDBC_TEMPLATE_URL));

    log.info("Loaded '{}' data store configuration defaults (Embedded H2)", storeName);

    return configuration;
  }
}
