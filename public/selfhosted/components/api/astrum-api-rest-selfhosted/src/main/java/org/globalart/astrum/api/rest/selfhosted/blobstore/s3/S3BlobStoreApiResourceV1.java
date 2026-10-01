
package org.globalart.astrum.api.rest.selfhosted.blobstore.s3;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.api.rest.common.blobstore.s3.S3BlobStoreApiUpdateValidation;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.crypto.secrets.SecretsFactory;

import static org.globalart.astrum.api.rest.selfhosted.blobstore.s3.S3BlobStoreApiResourceV1.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * v1 endpoint for S3 BlobStore REST API
 *
 * @since 3.24
 */
@Component
@Path(RESOURCE_URI)
public class S3BlobStoreApiResourceV1
    extends S3BlobStoreApiResource
{
  static final String RESOURCE_URI = V1_API_PREFIX + "/blobstores";

  @Autowired
  public S3BlobStoreApiResourceV1(
      final BlobStoreManager blobStoreManager,
      final S3BlobStoreApiUpdateValidation validation,
      final SecretsFactory secretsFactory,
      final ApplicationVersion applicationVersion)
  {
    super(blobStoreManager, validation, secretsFactory, applicationVersion);
  }
}
