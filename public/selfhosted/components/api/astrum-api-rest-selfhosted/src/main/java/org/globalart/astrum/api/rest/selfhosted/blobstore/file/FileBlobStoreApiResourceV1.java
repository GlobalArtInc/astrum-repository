
package org.globalart.astrum.api.rest.selfhosted.blobstore.file;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.blobstore.api.BlobStoreManager;

import static org.globalart.astrum.api.rest.selfhosted.blobstore.file.FileBlobStoreApiResourceV1.RESOURCE_URI;
import static org.globalart.astrum.rest.APIConstants.V1_API_PREFIX;
import org.springframework.stereotype.Component;

/**
 * v1 endpoint for File BlobStore REST API
 *
 * @since 3.24
 */
@Component
@Path(RESOURCE_URI)
public class FileBlobStoreApiResourceV1
    extends FileBlobStoreResource
{
  public static final String RESOURCE_URI = V1_API_PREFIX + "/blobstores";

  @Autowired
  public FileBlobStoreApiResourceV1(final BlobStoreManager blobStoreManager) {
    super(blobStoreManager);
  }
}
