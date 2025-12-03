
package org.globalart.astrum.ssl;

import org.globalart.astrum.ssl.spi.KeyStoreStorage;

public interface KeyStoreStorageFactory
{
  KeyStoreStorage create(String name);
}
