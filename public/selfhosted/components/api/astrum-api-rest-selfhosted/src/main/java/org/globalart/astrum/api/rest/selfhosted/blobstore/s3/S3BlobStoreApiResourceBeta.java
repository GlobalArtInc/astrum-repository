
package org.globalart.astrum.api.rest.selfhosted.blobstore.s3;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.api.rest.common.blobstore.s3.S3BlobStoreApiUpdateValidation;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.crypto.secrets.SecretsFactory;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.api.rest.selfhosted.blobstore.s3.S3BlobStoreApiResourceBeta.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.BETA_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * beta endpoint for S3 BlobStore REST API
 *
 * @since 3.24
 * @deprecated moving to {@link S3BlobStoreApiResourceV1}
 */
@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class S3BlobStoreApiResourceBeta
    extends S3BlobStoreApiResource
{
  static final String RESOURCE_URI = BETA_API_PREFIX + "/blobstores/s3";

  @Autowired
  public S3BlobStoreApiResourceBeta(
      final BlobStoreManager blobStoreManager,
      final S3BlobStoreApiUpdateValidation validation,
      final SecretsFactory secretsFactory,
      final ApplicationVersion applicationVersion)
  {
    super(blobStoreManager, validation, secretsFactory, applicationVersion);
  }
}
