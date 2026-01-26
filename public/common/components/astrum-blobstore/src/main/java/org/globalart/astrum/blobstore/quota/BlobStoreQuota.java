
package org.globalart.astrum.blobstore.quota;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;

/**
 * For a {@link BlobStore}, checks its usage against its quota
 *
 * @since 3.14
 */
public interface BlobStoreQuota
{
  /**
   * Ensure that the configuration has all the needed values
   * 
   * @param config - the configuration to be validated
   * @since 3.15
   */
  void validateConfig(BlobStoreConfiguration config);

  /**
   * @param blobStore - a blob store whose quota needs to be evaluated
   * @return {@link BlobStoreQuotaResult}
   */
  BlobStoreQuotaResult check(BlobStore blobStore);

  String getDisplayName();

  String getId();
}
