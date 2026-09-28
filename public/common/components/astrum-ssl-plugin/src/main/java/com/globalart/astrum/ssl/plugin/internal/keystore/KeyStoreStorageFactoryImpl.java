
package com.globalart.astrum.ssl.plugin.internal.keystore;

import org.globalart.astrum.ssl.KeyStoreStorageFactory;
import org.globalart.astrum.ssl.spi.KeyStoreStorage;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Qualifier(KeyStoreManagerImpl.NAME)
@Component
public class KeyStoreStorageFactoryImpl
    implements KeyStoreStorageFactory
{
  private final PersistentKeyStoreStorageManager keyStorageManager;

  @Autowired
  public KeyStoreStorageFactoryImpl(final PersistentKeyStoreStorageManager keyStorageManager) {
    this.keyStorageManager = checkNotNull(keyStorageManager);
  }

  @Override
  public KeyStoreStorage create(final String name) {
    return new KeyStoreStorageImpl(keyStorageManager, name);
  }
}
