
package org.globalart.astrum.blobstore;

import java.util.function.Supplier;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A builder for blob store configurations.
 *
 * @since 3.36
 */
public class BlobStoreConfigurationBuilder
{
  private final Supplier<BlobStoreConfiguration> configurationSupplier;

  private final String name;

  private String type;

  private String quotaType;

  private long quotaLimit;

  /**
   * Creates a new builder using the specified name for the resulting blob store.
   */
  public BlobStoreConfigurationBuilder(
      final String name,
      final Supplier<BlobStoreConfiguration> configurationSupplier)
  {
    this.name = checkNotNull(name);
    this.configurationSupplier = checkNotNull(configurationSupplier);
  }

  /**
   * Sets the type of blob store.
   */
  public BlobStoreConfigurationBuilder type(final String type) {
    this.type = checkNotNull(type);
    return this;
  }

  /**
   * Sets the blob store quota configuration.
   */
  public BlobStoreConfigurationBuilder quotaConfig(final String quotaType, final long limit) {
    this.quotaType = quotaType;
    this.quotaLimit = limit;
    return this;
  }

  /**
   * Creates the configuration for the desired blob store.
   */
  public BlobStoreConfiguration build() {
    final BlobStoreConfiguration configuration = configurationSupplier.get();
    configuration.setName(name);
    configuration.setType(type);
    if (quotaType != null) {
      configuration.attributes(BlobStoreQuotaSupport.ROOT_KEY).set(BlobStoreQuotaSupport.TYPE_KEY, quotaType);
      configuration.attributes(BlobStoreQuotaSupport.ROOT_KEY).set(BlobStoreQuotaSupport.LIMIT_KEY, quotaLimit);
    }
    return configuration;
  }
}
