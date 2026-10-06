
package org.globalart.astrum.self.hosted.node;

import java.io.File;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.ssl.KeyStoreStorageFactory;
import org.globalart.astrum.ssl.spi.KeyStoreStorage;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Implementation of {@link KeyStoreStorageFactory} for the node identity. Uses local filesystem as backing storage so
 * that node identity is specific to each node.
 *
 * @since 3.1
 */
@Component
@Qualifier(NodeKeyStoreManagerImpl.NAME)
public class FileKeyStoreStorageFactory
    implements KeyStoreStorageFactory
{
  private final File basedir;

  @Autowired
  public FileKeyStoreStorageFactory(final ApplicationDirectories directories) {
    this.basedir = new File(directories.getWorkDirectory("keystores"), NodeKeyStoreManagerImpl.NAME);
  }

  @Override
  public KeyStoreStorage create(final String keyStoreName) {
    checkNotNull(keyStoreName);
    return new FileKeyStoreStorage(new File(basedir, keyStoreName));
  }
}
