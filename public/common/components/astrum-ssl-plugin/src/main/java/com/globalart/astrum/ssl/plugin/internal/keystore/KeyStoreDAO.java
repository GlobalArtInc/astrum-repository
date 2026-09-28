
package com.globalart.astrum.ssl.plugin.internal.keystore;

import java.util.Optional;

import org.globalart.astrum.datastore.api.DataAccess;

/**
 * {@link KeyStoreData} access.
 *
 * @since 3.21
 */
public interface KeyStoreDAO
    extends DataAccess
{
  Optional<KeyStoreData> load(String name);

  boolean save(KeyStoreData entity);

  boolean delete(String name);

  Iterable<KeyStoreData> browse();
}
