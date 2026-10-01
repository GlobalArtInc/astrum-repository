
package org.globalart.astrum.api.rest.selfhosted.blobstore;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.api.rest.common.blobstore.model.BlobStoreQuotaResultXO;
import org.globalart.astrum.blobstore.ConnectionChecker;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaService;
import org.globalart.astrum.repository.blobstore.BlobStoreConfigurationStore;
import org.globalart.astrum.rest.WebApplicationMessageException;

import io.swagger.v3.oas.annotations.Hidden;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.api.rest.selfhosted.blobstore.BlobStoreResourceBeta.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.BETA_API_PREFIX;

/**
 * beta endpoint for BlobStore REST API
 *
 * @since 3.24
 * @deprecated moving to {@link BlobStoreResourceV1}
 */
@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class BlobStoreResourceBeta
    extends BlobStoreResource
{
  static final String RESOURCE_URI = BETA_API_PREFIX + "/blobstores";

  @Autowired
  public BlobStoreResourceBeta(
      final BlobStoreManager blobStoreManager,
      final BlobStoreConfigurationStore store,
      final BlobStoreQuotaService quotaService,
      final List<ConnectionChecker> connectionCheckers)
  {
    super(blobStoreManager, store, quotaService, connectionCheckers);
  }

  @Override
  @Deprecated
  public BlobStoreQuotaResultXO quotaStatus(final String name) {
    throw new WebApplicationMessageException(Status.BAD_REQUEST, "not supported");
  }
}
