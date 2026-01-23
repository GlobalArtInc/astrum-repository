
package org.globalart.astrum.blobstore;

import java.io.InputStream;
import java.nio.file.Path;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobSession;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.transaction.Transaction;
import org.globalart.astrum.transaction.TransactionSupport;

import com.google.common.hash.HashCode;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Simple in-memory {@link BlobSession}.
 *
 * @since 3.20
 */
public class MemoryBlobSession
    extends TransactionSupport
    implements BlobSession<Transaction>
{
  private static final Logger log = LoggerFactory.getLogger(MemoryBlobSession.class);

  private final BlobStore blobStore;

  private final Set<BlobId> creates = new HashSet<>();

  private final Set<BlobId> deletes = new HashSet<>();

  public MemoryBlobSession(final BlobStore blobStore) {
    this.blobStore = checkNotNull(blobStore);
  }

  @Override
  public Transaction getTransaction() {
    return this;
  }

  @Override
  public Blob create(final InputStream blobData, final Map<String, String> headers, final BlobId blobId) {
    Blob blob = blobStore.create(blobData, headers, blobId);
    creates.add(blob.getId());
    return blob;
  }

  @Override
  public Blob create(final Path sourceFile, final Map<String, String> headers, final long size, final HashCode sha1) {
    Blob blob = blobStore.create(sourceFile, headers, size, sha1);
    creates.add(blob.getId());
    return blob;
  }

  @Override
  public Blob copy(final BlobId blobId, final Map<String, String> headers) {
    Blob blob = blobStore.copy(blobId, headers);
    creates.add(blob.getId());
    return blob;
  }

  @Override
  public Blob get(final BlobId blobId, final boolean includeDeleted) {
    return includeDeleted || !deletes.contains(blobId) ? blobStore.get(blobId) : null;
  }

  @Override
  public boolean exists(final BlobId blobId) {
    return blobStore.exists(blobId);
  }

  @Override
  public boolean delete(final BlobId blobId) {
    return deletes.add(blobId);
  }

  @Override
  protected void doCommit() {
    deleteChangeSet(deletes, "committing " + reason());
    resetState();
  }

  @Override
  protected void doRollback() {
    deleteChangeSet(creates, "rolling back " + reason());
    resetState();
  }

  @Override
  public void close() {
    if (!creates.isEmpty() || !deletes.isEmpty()) {
      log.warn("Uncommitted changes on close");
      rollback(); // match data-store behaviour: roll back uncommitted changes on close
    }
  }

  private void deleteChangeSet(final Set<BlobId> changeSet, final String reason) {
    for (BlobId blobId : changeSet) {
      try {
        blobStore.delete(blobId, reason);
      }
      catch (Throwable e) { // NOSONAR: ignore all errors during commit/rollback
        // ...because we can't roll back any associated DB changes at this point
        log.warn("Problem deleting {}:{} while {}", storeName(), blobId, reason, e);
      }
    }
  }

  private String storeName() {
    try {
      return blobStore.getBlobStoreConfiguration().getName();
    }
    catch (Throwable e) { // NOSONAR: don't fail when logging with broken config
      return "<unknown>";
    }
  }

  private void resetState() {
    creates.clear();
    deletes.clear();
  }
}
