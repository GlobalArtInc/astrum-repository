
package org.globalart.astrum.repository.view.payloads;

import java.io.Closeable;
import java.io.InputStream;
import java.util.Map;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobRef;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreException;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.common.io.InputStreamSupplier;

import com.google.common.hash.HashCode;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Blob handle that holds information for a temporary blob used in place of temporary files and streams. Instances must
 * be closed by the caller.
 *
 * @since 3.1
 */
public class TempBlob
    implements Closeable, InputStreamSupplier
{
  private static final Logger log = LoggerFactory.getLogger(TempBlob.class);

  protected final Blob blob;

  protected final Map<HashAlgorithm, HashCode> hashes;

  protected final boolean hashesVerified;

  protected final BlobStore blobStore;

  protected boolean deleted = false;

  public TempBlob(
      final Blob blob,
      final Map<HashAlgorithm, HashCode> hashes,
      final boolean hashesVerified,
      final BlobStore blobStore)
  {
    this.blob = checkNotNull(blob);
    this.hashes = checkNotNull(hashes);
    this.hashesVerified = hashesVerified;
    this.blobStore = checkNotNull(blobStore);
  }

  /**
   * The actual blob this instance is pointing to.
   */
  public Blob getBlob() {
    return blob;
  }

  public BlobRef getBlobRef(final String node) {
    return new BlobRef(
        node,
        blobStore.getBlobStoreConfiguration().getName(),
        getBlob().getId().asUniqueString());
  }

  /**
   * Exact hashes for the blob. Typically calculated by storage subsystem while blob was getting saved, but sometimes
   * provided based on precalculated or known values.
   */
  public Map<HashAlgorithm, HashCode> getHashes() {
    return hashes;
  }

  /**
   * Returns a boolean indicating whether the hashes associated with this blob have been verified.
   */
  public boolean getHashesVerified() {
    return hashesVerified;
  }

  @Override
  public void close() {
    if (deleted) {
      return;
    }
    try {
      blobStore.deleteIfTemp(blob);
      deleted = true;
    }
    catch (BlobStoreException e) {
      log.debug("Unable to delete blob {} in blob store {}", blob.getId(),
          blobStore.getBlobStoreConfiguration().getName(), e);
    }
  }

  @Override
  public InputStream get() {
    return blob.getInputStream();
  }
}
