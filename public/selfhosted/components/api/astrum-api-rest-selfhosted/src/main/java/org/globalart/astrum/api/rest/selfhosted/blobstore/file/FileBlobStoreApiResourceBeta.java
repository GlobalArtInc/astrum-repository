
package org.globalart.astrum.api.rest.selfhosted.blobstore.file;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.blobstore.api.BlobStoreManager;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.api.rest.selfhosted.blobstore.file.FileBlobStoreApiResourceBeta.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.BETA_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * beta endpoint for File BlobStore REST API
 *
 * @since 3.24
 * @deprecated moving to {@link FileBlobStoreApiResourceV1}
 */
@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class FileBlobStoreApiResourceBeta
    extends FileBlobStoreResource
{
  static final String RESOURCE_URI = BETA_API_PREFIX + "/blobstores/file";

  @Autowired
  public FileBlobStoreApiResourceBeta(final BlobStoreManager blobStoreManager) {
    super(blobStoreManager);
  }
}
