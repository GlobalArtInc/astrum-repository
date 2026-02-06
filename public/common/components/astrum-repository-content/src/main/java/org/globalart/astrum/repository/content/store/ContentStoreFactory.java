
package org.globalart.astrum.repository.content.store;

import org.globalart.astrum.datastore.api.ContentDataAccess;

/**
 * Assisted-Inject factory template to help create format-specific store instances in different data stores.
 *
 * The {@link STORE} type-parameter tells Assisted-Inject what type of store to create while the {@link DAO}
 * type-parameter helps it match the right {@link ContentStoreSupport} signature when generating the factory.
 * The format-specific DAO class is also passed into the factory at creation time because we can't reliably
 * extract it from the store implementation when the format re-uses the default STORE types.
 *
 * @see FormatStoreFactory
 *
 * @since 3.26
 */
interface ContentStoreFactory<STORE extends ContentStoreSupport<DAO>, DAO extends ContentDataAccess>
{
  /**
   * Creates a format-specific store instance for the given data store.
   *
   * @param contentStoreName The content data store
   * @param formatDaoClass The format-specific DAO class
   */
  STORE createContentStore(String contentStoreName, Class<DAO> formatDaoClass);
}
