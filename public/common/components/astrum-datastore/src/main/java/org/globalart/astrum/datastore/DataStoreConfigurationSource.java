
package org.globalart.astrum.datastore;

import org.globalart.astrum.datastore.api.DataStoreConfiguration;

import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;

/**
 * Source of {@link DataStoreConfiguration}s.
 *
 * @since 3.19
 */
public interface DataStoreConfigurationSource
{
  /**
   * A user friendly name for this source to be presented in the UI.
   */
  String getName();

  /**
   * Is this source currently enabled to supply configurations?
   */
  default boolean isEnabled() {
    return true;
  }

  /**
   * Can this source also store new/updated configurations?
   */
  default boolean isModifiable() {
    return false;
  }

  /**
   * Browse the names of data stores configured by this source.
   */
  Iterable<String> browseStoreNames();

  /**
   * Load the configuration for the named data store from this source.
   */
  DataStoreConfiguration load(String storeName);

  /**
   * Save the given data store configuration using this source.
   */
  default void save(DataStoreConfiguration configuration) {
    // no-op by default for non-modifiable sources
  }

  /**
   * Delete the given data store configuration from this source.
   */
  default void delete(DataStoreConfiguration configuration) {
    // no-op by default for non-modifiable sources
  }

  /**
   * Defines the {@link Order} for using environment variables as a configuration source
   */
  static final int ORDER_ENVIRONMENT_VALUES = Ordered.HIGHEST_PRECEDENCE;

  /**
   * Defines the {@link Order} for using system properties as a configuration source
   */
  static final int ORDER_SYSTEM_PROPERTIES = ORDER_ENVIRONMENT_VALUES + 10;

  /**
   * Defines the {@link Order} for using file system as a configuration source
   */
  static final int ORDER_FILE = ORDER_SYSTEM_PROPERTIES + 10;

  /**
   * Defines the {@link Order} for using the default source
   */
  static final int ORDER_DEFAULT_SOURCE = Ordered.LOWEST_PRECEDENCE;
}
