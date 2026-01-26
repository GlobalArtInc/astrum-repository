
package org.globalart.astrum.blobstore.quota;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;

/**
 * For a {@link BlobStore}, checks it usage against its quotas
 *
 * @since 3.14
 */
public interface BlobStoreQuotaService
{
  /**
   * If the config has a quota, ensure that the configuration has all the needed values
   * 
   * @param config - the configuration to be validated
   * @since 3.15
   */
  void validateSoftQuotaConfig(BlobStoreConfiguration config);

  /**
   * @param blobStore - a blob store whose quota needs to be evaluated
   * @return null if the blob store doesn't have a quota otherwise return a {@link BlobStoreQuotaResult}
   */
  BlobStoreQuotaResult checkQuota(BlobStore blobStore);
}
