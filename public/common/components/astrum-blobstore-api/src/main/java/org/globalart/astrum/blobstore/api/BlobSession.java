
package org.globalart.astrum.blobstore.api;

import java.io.InputStream;
import java.nio.file.Path;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.transaction.Transaction;
import org.globalart.astrum.transaction.TransactionalSession;

import com.google.common.hash.HashCode;

/**
 * Represents a session with a {@link BlobStore}.
 *
 * @since 3.20
 */
public interface BlobSession<T extends Transaction>
    extends TransactionalSession<T>
{
  /**
   * @see BlobStore#create(InputStream, Map)
   */
  default Blob create(InputStream blobData, Map<String, String> headers) {
    return create(blobData, headers, null);
  }

  /**
   * @see BlobStore#create(InputStream, Map, BlobId)
   */
  Blob create(InputStream blobData, Map<String, String> headers, @Nullable BlobId blobId);

  /**
   * @see BlobStore#create(Path, Map, long, HashCode)
   */
  Blob create(Path sourceFile, Map<String, String> headers, long size, HashCode sha1);

  /**
   * @see BlobStore#copy(BlobId, Map)
   */
  Blob copy(BlobId blobId, Map<String, String> headers);

  /**
   * @see BlobStore#get(BlobId)
   */
  @Nullable
  default Blob get(BlobId blobId) {
    return get(blobId, false);
  }

  /**
   * @see BlobStore#get(BlobId, boolean)
   */
  @Nullable
  Blob get(BlobId blobId, boolean includeDeleted);

  /**
   * @see BlobStore#exists(BlobId)
   */
  boolean exists(BlobId blobId);

  /**
   * @see BlobStore#delete(BlobId, String)
   */
  boolean delete(BlobId blobId);
}
