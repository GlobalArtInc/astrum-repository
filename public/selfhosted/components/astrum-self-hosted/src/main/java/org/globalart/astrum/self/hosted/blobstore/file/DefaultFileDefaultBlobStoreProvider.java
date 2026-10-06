
package org.globalart.astrum.self.hosted.blobstore.file;

import java.util.function.Supplier;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.DefaultBlobStoreProvider;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.blobstore.api.BlobStoreManager.DEFAULT_BLOBSTORE_NAME;

/**
 * Provider for creating the default browse node configuration
 *
 * @since 3.37
 */
@Primary
@Component
@Qualifier("default")
public class DefaultFileDefaultBlobStoreProvider
    implements DefaultBlobStoreProvider
{
  @Override
  public BlobStoreConfiguration get(final Supplier<BlobStoreConfiguration> configurationSupplier) {
    return new FileBlobStoreConfigurationBuilder(DEFAULT_BLOBSTORE_NAME, configurationSupplier).build();
  }
}
