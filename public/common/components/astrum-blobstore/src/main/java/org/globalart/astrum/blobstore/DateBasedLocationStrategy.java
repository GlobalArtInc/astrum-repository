
package org.globalart.astrum.blobstore;

import java.time.OffsetDateTime;

import org.globalart.astrum.blobstore.api.BlobId;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.blobstore.api.BlobRef.DATE_TIME_PATH_FORMATTER;

/**
 * Stores blobs in a date-based directory tree.
 * <p>
 * For example, {@code yyyy/mm/dd/hours/minutes/blobId}
 */
public class DateBasedLocationStrategy
    extends LocationStrategySupport
{
  @Override
  public String location(final BlobId blobId) {
    checkNotNull(blobId);

    OffsetDateTime blobCreationTime = blobId.getBlobCreatedRef();
    String datePath = blobCreationTime.format(DATE_TIME_PATH_FORMATTER);
    return datePath + "/" + escapeFilename(blobId.asUniqueString());
  }
}
