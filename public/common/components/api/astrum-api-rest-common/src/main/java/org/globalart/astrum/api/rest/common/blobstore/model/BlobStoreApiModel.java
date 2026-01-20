
package org.globalart.astrum.api.rest.common.blobstore.model;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport;

import io.swagger.v3.oas.annotations.media.Schema;

import static org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport.LIMIT_KEY;
import static org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport.ROOT_KEY;
import static org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport.TYPE_KEY;

/**
 * @since 3.19
 */
public abstract class BlobStoreApiModel
{
  @Schema(description = "Settings to control the soft quota")
  private BlobStoreApiSoftQuota softQuota;

  public BlobStoreApiModel() {
  }

  public BlobStoreApiModel(BlobStoreConfiguration configuration) {
    softQuota = createSoftQuota(configuration);
  }

  public BlobStoreApiSoftQuota getSoftQuota() {
    return softQuota;
  }

  public void setSoftQuota(final BlobStoreApiSoftQuota softQuota) {
    this.softQuota = softQuota;
  }

  public BlobStoreConfiguration toBlobStoreConfiguration(final BlobStoreConfiguration configuration) {
    setSoftQuotaAttributes(configuration);
    return configuration;
  }

  private void setSoftQuotaAttributes(BlobStoreConfiguration configuration) {
    if (softQuota == null) {
      return;
    }

    configuration.attributes(ROOT_KEY).set(TYPE_KEY, softQuota.getType());
    if (softQuota.getLimit() == null) {
      configuration.attributes(ROOT_KEY).set(LIMIT_KEY, -1l);
    }
    else {
      configuration.attributes(ROOT_KEY).set(LIMIT_KEY, softQuota.getLimit());
    }
  }

  private BlobStoreApiSoftQuota createSoftQuota(BlobStoreConfiguration configuration) {
    if (configuration.attributes(ROOT_KEY).isEmpty()) {
      return null;
    }

    BlobStoreApiSoftQuota newSoftQuota = new BlobStoreApiSoftQuota();
    newSoftQuota.setType(BlobStoreQuotaSupport.getType(configuration));
    newSoftQuota.setLimit(BlobStoreQuotaSupport.getLimit(configuration));
    return newSoftQuota;
  }
}
