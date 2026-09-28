
package com.globalart.astrum.ssl.plugin.internal.keystore;

import java.util.List;
import java.util.Optional;

/**
 * Manager persistent storage for key stores
 */
public interface PersistentKeyStoreStorageManager
{
  boolean exists(final String keyStoreName);

  Optional<KeyStoreData> load(final String keyStoreName);

  void save(final KeyStoreData data);

  List<KeyStoreData> browse();
}
