
package org.globalart.astrum.blobstore.api;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link BlobRef} which carries extra information about the associated Blob
 */
public class HeavyBlobRef
    extends BlobRef
{
  private BlobMetrics metrics;

  public HeavyBlobRef(final BlobRef blobref, final BlobMetrics metrics) {
    super(checkNotNull(blobref).getNode(), blobref.getStore(), blobref.getBlob(), blobref.getDateBasedRef());
    this.metrics = checkNotNull(metrics);
  }

  public BlobMetrics getMetrics() {
    return metrics;
  }
}
