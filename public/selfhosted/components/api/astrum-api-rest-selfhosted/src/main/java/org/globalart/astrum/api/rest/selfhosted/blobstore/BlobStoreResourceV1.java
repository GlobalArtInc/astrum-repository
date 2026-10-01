
package org.globalart.astrum.api.rest.selfhosted.blobstore;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.blobstore.ConnectionChecker;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaService;
import org.globalart.astrum.repository.blobstore.BlobStoreConfigurationStore;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.api.rest.selfhosted.blobstore.BlobStoreResourceV1.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;

/**
 * v1 endpoint for BlobStore REST API
 *
 * @since 3.24
 */
@Component
@Path(RESOURCE_URI)
public class BlobStoreResourceV1
    extends BlobStoreResource
{
  static final String RESOURCE_URI = V1_API_PREFIX + "/blobstores";

  @Autowired
  public BlobStoreResourceV1(
      final BlobStoreManager blobStoreManager,
      final BlobStoreConfigurationStore store,
      final BlobStoreQuotaService quotaService,
      final List<ConnectionChecker> connectionCheckers)
  {
    super(blobStoreManager, store, quotaService, connectionCheckers);
  }
}
