
package org.globalart.astrum.datastore;

import org.globalart.astrum.datastore.api.DataStoreConfiguration;

/**
 * Restores DataStores from backup.
 *
 * @since 3.21
 */
public interface DataStoreRestorer
{
  boolean maybeRestore(DataStoreConfiguration dataStoreConfiguration);
}
