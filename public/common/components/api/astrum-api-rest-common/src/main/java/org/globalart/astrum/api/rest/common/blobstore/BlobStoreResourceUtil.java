
package org.globalart.astrum.api.rest.common.blobstore;

import org.globalart.astrum.rest.WebApplicationMessageException;

import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.Response.Status.BAD_REQUEST;
import static jakarta.ws.rs.core.Response.Status.NOT_FOUND;

/**
 * @since 3.19
 */
public class BlobStoreResourceUtil
{
  /**
   * Throws {@link BAD_REQUEST} exception in case when BlobStore manager could not perform operation (for example,
   * blobstore is in use)
   *
   * @param message error message
   * @throws WebApplicationMessageException
   */
  public static void throwBlobStoreBadRequestException(final String message) throws WebApplicationMessageException {
    throw new WebApplicationMessageException(
        BAD_REQUEST,
        "\"" + message + "\"",
        APPLICATION_JSON);
  }

  /**
   * Returns a {@link NOT_FOUND} WebApplicationMessageException when blobstore is not found.
   *
   * @param blobStoreType The type of the blobstore (e.g.: File, Group, S3, Azure Cloud Storage).
   * @param blobStoreName The name of the blobstore.
   * @return {@link WebApplicationMessageException}.
   */
  public static WebApplicationMessageException createBlobStoreNotFoundException(
      final String blobStoreType,
      final String blobStoreName)
  {
    return new WebApplicationMessageException(
        NOT_FOUND,
        String.format("Unable to find %s '%s' blobstore", blobStoreType, blobStoreName),
        APPLICATION_JSON);
  }

  /**
   * Throws a {@link NOT_FOUND} WebApplicationMessageException when blobstore is not found.
   *
   * @param blobStoreType The type of the blobstore (e.g.: File, Group, S3, Azure Cloud Storage).
   * @param blobStoreName The name of the blobstore.
   * @throws {@link WebApplicationMessageException}.
   */
  public static void throwCreateBlobStoreNotFoundException(
      final String blobStoreType,
      final String blobStoreName)
  {
    throw createBlobStoreNotFoundException(blobStoreType, blobStoreName);
  }
}
