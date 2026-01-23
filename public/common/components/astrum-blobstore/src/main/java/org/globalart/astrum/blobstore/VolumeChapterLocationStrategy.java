
package org.globalart.astrum.blobstore;

import org.globalart.astrum.blobstore.api.BlobId;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Stores blobs in a two-deep directory tree.
 *
 * The first layer, {@code vol}, having {@link #TIER_1_MODULO} directories,
 * and the second {@code chap} having {@link #TIER_2_MODULO}.
 *
 * @since 3.0
 */
public class VolumeChapterLocationStrategy
    extends LocationStrategySupport
{
  private static final int TIER_1_MODULO = 43;

  private static final int TIER_2_MODULO = 47;

  @Override
  public String location(final BlobId blobId) {
    checkNotNull(blobId);

    return String.format("vol-%02d/chap-%02d/%s",
        tier(blobId, TIER_1_MODULO),
        tier(blobId, TIER_2_MODULO),
        escapeFilename(blobId.asUniqueString()));
  }

  private int tier(final BlobId blobId, final int modulo) {
    return Math.abs(blobId.hashCode() % modulo) + 1;
  }
}
