
package org.globalart.astrum.blobstore.api;

import java.io.Serializable;
import java.time.OffsetDateTime;
import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A unique identifier for a blob within a specific {@link BlobStore}.
 *
 * @since 3.0
 */
public class BlobId
    implements Serializable, Comparable<BlobId>
{
  private final String id;

  private final OffsetDateTime blobCreatedRef;

  private final String blobstorePath;

  /**
   * @deprecated Use {@link #BlobId(String, OffsetDateTime)} instead.
   */
  @Deprecated
  public BlobId(final String id) {
    this(id, null);
  }

  public BlobId(final String id, @Nullable OffsetDateTime blobCreatedRef) {
    this.id = checkNotNull(id);
    this.blobCreatedRef = blobCreatedRef;
    this.blobstorePath = null;
  }

  public BlobId(final String id, @Nullable OffsetDateTime blobCreatedRef, @Nullable String blobstorePath) {
    this.id = checkNotNull(id);
    this.blobCreatedRef = blobCreatedRef;
    this.blobstorePath = blobstorePath;
  }

  public String asUniqueString() {
    return id;
  }

  public OffsetDateTime getBlobCreatedRef() {
    return blobCreatedRef;
  }

  public String getBlobstorePath() {
    return blobstorePath;
  }

  @Override
  public String toString() {
    return id;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }

    BlobId blobId = (BlobId) o;

    return id.equals(blobId.id);
  }

  @Override
  public int hashCode() {
    return id.hashCode();
  }

  @Override
  public int compareTo(final BlobId o) {
    return id.compareTo(o.id);
  }
}
