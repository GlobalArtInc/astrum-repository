
package org.globalart.astrum.blobstore;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaService;

public abstract class BlobStoreDescriptorSupport
    implements BlobStoreDescriptor
{

  private final BlobStoreQuotaService quotaService;

  public BlobStoreDescriptorSupport(final BlobStoreQuotaService quotaService) {
    this.quotaService = quotaService;
  }

  @Override
  public void validateConfig(final BlobStoreConfiguration configuration) {
    quotaService.validateSoftQuotaConfig(configuration);
  }
}
