
package org.globalart.astrum.self.hosted.blobstore.file;

import java.util.function.Supplier;

import org.globalart.astrum.blobstore.BlobStoreConfigurationBuilder;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A builder to configure a file blob store.
 *
 * @since 3.1
 */
public class FileBlobStoreConfigurationBuilder
    extends BlobStoreConfigurationBuilder
{
  private static final String CONFIG_KEY = "file";

  private static final String PATH_KEY = "path";

  private static final String TYPE = "File";

  private String path;

  /**
   * Creates a new builder using the specified name for the resulting blob store. Unless customized, the name is also
   * used as the path for the blob store.
   */
  public FileBlobStoreConfigurationBuilder(
      final String name,
      final Supplier<BlobStoreConfiguration> configurationSupplier)
  {
    super(name, configurationSupplier);
    type(TYPE);
    this.path = name;
  }

  /**
   * Sets the local path where the blobs are persisted.
   */
  public FileBlobStoreConfigurationBuilder path(final String path) {
    this.path = checkNotNull(path);
    return this;
  }

  /**
   * Creates the configuration for the desired file blob store.
   */
  @Override
  public BlobStoreConfiguration build() {
    final BlobStoreConfiguration configuration = super.build();
    configuration.attributes(CONFIG_KEY).set(PATH_KEY, path);
    return configuration;
  }
}
