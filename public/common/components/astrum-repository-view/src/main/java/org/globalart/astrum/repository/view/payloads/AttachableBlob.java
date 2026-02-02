
package org.globalart.astrum.repository.view.payloads;

import java.util.Map;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.common.hash.HashAlgorithm;

import com.google.common.hash.HashCode;

/**
 * A {@link TempBlob} which represents an ingested {@link Blob} that as become detached from its asset. When Astrum is
 * using SQL mode there is special handling which can attempt to re-attach the blob rather than copy it.
 */
public class AttachableBlob
    extends TempBlob
{
  private boolean attached = false;

  public AttachableBlob(
      final Blob blob,
      final Map<HashAlgorithm, HashCode> hashes,
      final boolean hashesVerified,
      final BlobStore blobStore)
  {
    super(blob, hashes, hashesVerified, blobStore);
  }

  /**
   * Marks this instances as having been attached.
   */
  public void markAttached() {
    this.attached = true;
  }

  /**
   * Check whether the blob has been attached since this was called.
   */
  public boolean isAttached() {
    return attached;
  }

  @Override
  public void close() {
    // no-op
  }
}
