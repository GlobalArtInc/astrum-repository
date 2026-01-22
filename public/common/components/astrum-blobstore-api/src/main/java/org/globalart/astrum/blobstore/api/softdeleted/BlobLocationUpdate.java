
package org.globalart.astrum.blobstore.api.softdeleted;

import java.time.OffsetDateTime;

/**
 * Data transfer record for batch updating blob location in soft_deleted_blobs table.
 * Contains the blob ID string, new source blob store name, and new date path reference.
 */
public record BlobLocationUpdate(
    String blobId,
    String newSourceBlobStoreName,
    OffsetDateTime newDatePathRef)
{
}
