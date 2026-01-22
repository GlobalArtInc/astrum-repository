
package org.globalart.astrum.blobstore.api.softdeleted;

import java.time.OffsetDateTime;

import org.globalart.astrum.common.entity.ContinuationAware;

/**
 * Data object for the soft deleted blobs table.
 */
public interface SoftDeletedBlob
    extends ContinuationAware
{
  String getSourceBlobStoreName();

  OffsetDateTime getDeletedDate();

  String getBlobId();

  OffsetDateTime getDatePathRef();
}
