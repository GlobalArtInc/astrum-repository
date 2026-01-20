

package org.globalart.astrum.api.rest.common.blobstore.s3;

/**
 * Constants for S3 blobstore REST APIs.
 */
public class S3BlobStoreApiConstants
{
  private S3BlobStoreApiConstants() {
  }

  public static final String NOT_AN_S3_BLOB_STORE_MSG_FORMAT = "\"%s is not an S3 blob store.\"";

  public static final String BLOB_STORE_NAME_UPDATE_ERROR_MESSAGE = "Renaming an S3 blob store name is not supported";

  public static final String NON_EXISTENT_BLOB_STORE_ERROR_MESSAGE_FORMAT = "No S3 blob store called '%s'";

  public static final String BLOB_STORE_TYPE_MISMATCH_ERROR_FORMAT = "Blob store %s is not an S3 blob store";

  static final String DUPLICATE_REGIONS_ERROR_MESSAGE = "More than one failover bucket defined for a single region.";

  static final String FAILOVER_DEFAULT_ERROR_MESSAGE = "Failovers may not contain 'default' as a region";

  static final String MATCHES_PRIMARY_ERROR_MESSAGE = "Failover region duplicates primary region";

}
