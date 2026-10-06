
package org.globalart.astrum.self.hosted.node;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.crypto.CryptoHelper;
import org.globalart.astrum.ssl.KeyStoreManager;
import org.globalart.astrum.ssl.KeyStoreManagerConfiguration;
import org.globalart.astrum.ssl.KeyStoreStorageFactory;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * Node {@link KeyStoreManager}.
 *
 * @since 3.0
 */
@Component
@Qualifier(NodeKeyStoreManagerImpl.NAME)
public class NodeKeyStoreManagerImpl
    extends org.globalart.astrum.ssl.KeyStoreManagerImpl
{
  public static final String NAME = "node";

  @Autowired
  public NodeKeyStoreManagerImpl(
      final CryptoHelper crypto,
      @Qualifier(NAME) final KeyStoreStorageFactory storageManager,
      @Qualifier(NAME) final KeyStoreManagerConfiguration config)
  {
    super(crypto, storageManager, config);
  }
}
