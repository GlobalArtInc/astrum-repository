
package org.globalart.astrum.blobstore.internal.softdeleted;

import java.time.OffsetDateTime;

import org.globalart.astrum.blobstore.api.softdeleted.SoftDeletedBlob;

/**
 * Data object for the soft deleted blobs table.
 */
public class SoftDeletedBlobsData
    implements SoftDeletedBlob
{
  Integer recordId; // NOSONAR: internal id

  private String blobId;

  private String sourceBlobStoreName;

  private OffsetDateTime deletedDate;

  private OffsetDateTime datePathRef;

  @Override
  public String getSourceBlobStoreName() {
    return sourceBlobStoreName;
  }

  public void setSourceBlobStoreName(final String sourceBlobStoreName) {
    this.sourceBlobStoreName = sourceBlobStoreName;
  }

  @Override
  public OffsetDateTime getDeletedDate() {
    return deletedDate;
  }

  public void setDeletedDate(final OffsetDateTime deletedDate) {
    this.deletedDate = deletedDate;
  }

  @Override
  public String getBlobId() {
    return blobId;
  }

  @Override
  public OffsetDateTime getDatePathRef() {
    return datePathRef;
  }

  public void setDatePathRef(final OffsetDateTime datePathRef) {
    this.datePathRef = datePathRef;
  }

  @Override
  public String nextContinuationToken() {
    return Integer.toString(recordId);
  }
}
