
package org.globalart.astrum.blobstore.api;

/**
 * @since 3.4
 */
public final class BlobAttributesConstants
{
  public static final String HEADER_PREFIX = "@";

  public static final String SHA1_HASH_ATTRIBUTE = "sha1";

  public static final String CONTENT_SIZE_ATTRIBUTE = "size";

  public static final String CREATION_TIME_ATTRIBUTE = "creationTime";

  public static final String DELETED_ATTRIBUTE = "deleted";

  public static final String DELETED_REASON_ATTRIBUTE = "deletedReason";

  public static final String DELETED_DATETIME_ATTRIBUTE = "deletedDateTime";

  public static final String ASSET_ATTRIBUTE_HEADER_PREFIX = "attributes.asset.";

  public static final String COMPONENT_ATTRIBUTE_HEADER_PREFIX = "attributes.component.";

  public static final String LAST_DOWNLOADED_ATTRIBUTE = "lastDownloaded";

  public static final String ORIGINAL_LOCATION_ATTRIBUTE = "originalLocation";

  public static final String SOFT_DELETED_LOCATION_ATTRIBUTE = "softDeletedLocation";

  private BlobAttributesConstants() {
  }
}
