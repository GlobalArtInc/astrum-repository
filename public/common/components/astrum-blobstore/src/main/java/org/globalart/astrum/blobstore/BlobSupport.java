
package org.globalart.astrum.blobstore;

import java.io.BufferedInputStream;
import java.io.InputStream;
import java.util.Map;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

import org.globalart.astrum.common.concurrent.Locks;
import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobMetrics;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Support for {@link Blob} implementations with support for locking.
 *
 * @since 3.3
 */
public abstract class BlobSupport
    implements Blob
{
  private final BlobId blobId;

  private final Lock lock;

  private Map<String, String> headers;

  private BlobMetrics metrics;

  private volatile boolean stale;

  public BlobSupport(final BlobId blobId) {
    this.blobId = checkNotNull(blobId);
    lock = new ReentrantLock();
    stale = true;
  }

  public void refresh(final Map<String, String> headers, final BlobMetrics metrics) {
    this.headers = checkNotNull(headers);
    this.metrics = checkNotNull(metrics);
    stale = false;
  }

  public void markStale() {
    stale = true;
  }

  public boolean isStale() {
    return stale;
  }

  @Override
  public BlobId getId() {
    return blobId;
  }

  @Override
  public Map<String, String> getHeaders() {
    return headers;
  }

  @Override
  public BlobMetrics getMetrics() {
    return metrics;
  }

  public Lock lock() {
    return Locks.lock(lock);
  }

  @Override
  public InputStream getInputStream() {
    InputStream inputStream = doGetInputStream();
    if (!inputStream.markSupported()) {
      return new BufferedInputStream(inputStream);
    }
    return inputStream;
  }

  /**
   * Gets the natural input stream for the given blob
   *
   * @since 3.19
   */
  protected abstract InputStream doGetInputStream();
}
