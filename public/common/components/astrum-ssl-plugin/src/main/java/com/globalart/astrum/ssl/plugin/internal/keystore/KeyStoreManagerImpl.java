
package com.globalart.astrum.ssl.plugin.internal.keystore;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.crypto.CryptoHelper;
import org.globalart.astrum.ssl.KeyStoreManagerConfiguration;
import org.globalart.astrum.ssl.KeyStoreStorageFactory;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

/**
 * SSL plugin specific key-store manager.
 *
 * @since ssl 1.0
 */
@Lazy
@Component
@Qualifier(KeyStoreManagerImpl.NAME)
public class KeyStoreManagerImpl
    extends org.globalart.astrum.ssl.KeyStoreManagerImpl
{
  public static final String NAME = "ssl";

  @Autowired
  public KeyStoreManagerImpl(
      final CryptoHelper crypto,
      @Qualifier(NAME) final KeyStoreStorageFactory storageFactory,
      @Qualifier(NAME) final KeyStoreManagerConfiguration config)
  {
    super(crypto, storageFactory, config);
  }
}
