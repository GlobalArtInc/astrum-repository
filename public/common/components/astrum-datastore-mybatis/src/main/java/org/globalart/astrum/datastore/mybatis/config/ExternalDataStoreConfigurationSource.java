package org.globalart.astrum.datastore.mybatis.config;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

import org.globalart.astrum.datastore.DataStoreConfigurationSource;
import org.globalart.astrum.datastore.api.DataStoreConfiguration;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Strings.isNullOrEmpty;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

public abstract class ExternalDataStoreConfigurationSource
    implements DataStoreConfigurationSource
{
  static final String JDBC_URL = "jdbcUrl";

  static final List<String> ATTRIBUTES = List.of(JDBC_URL, "username", "password", "schema", "maximumPoolSize", "advanced");

  protected final Logger log = LoggerFactory.getLogger(getClass());

  protected abstract String qualifier();

  protected abstract String lookup(String storeName, String attribute);

  @Override
  public boolean isEnabled() {
    return !isNullOrEmpty(lookup(DEFAULT_DATASTORE_NAME, JDBC_URL));
  }

  @Override
  public Iterable<String> browseStoreNames() {
    return isEnabled() ? List.of(DEFAULT_DATASTORE_NAME) : List.of();
  }

  @Override
  public DataStoreConfiguration load(final String storeName) {
    checkArgument(DEFAULT_DATASTORE_NAME.equalsIgnoreCase(storeName),
        "%s is not valid, %s is the only valid data store name", storeName, DEFAULT_DATASTORE_NAME);
    Map<String, String> attributes = new LinkedHashMap<>();
    for (String attribute : ATTRIBUTES) {
      String value = lookup(DEFAULT_DATASTORE_NAME, attribute);
      if (!isNullOrEmpty(value)) {
        attributes.put(attribute, value.trim());
      }
    }
    DataStoreConfiguration configuration = new DataStoreConfiguration();
    configuration.setName(DEFAULT_DATASTORE_NAME);
    configuration.setType("jdbc");
    configuration.setSource(qualifier());
    configuration.setAttributes(attributes);
    log.info("Loaded '{}' data store configuration from {}", storeName, getName());
    return configuration;
  }

  static String environmentKey(final String storeName, final String attribute) {
    return ("ASTRUM_DATASTORE_" + storeName + "_" + attribute).toUpperCase(Locale.ROOT);
  }

  static String propertyKey(final String storeName, final String attribute) {
    return "astrum.datastore." + storeName + "." + attribute;
  }
}
