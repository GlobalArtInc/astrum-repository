
package org.globalart.astrum.blobstore.metrics;

import java.io.IOException;
import java.util.EnumMap;
import java.util.Map;
import java.util.stream.Stream;

import org.globalart.astrum.blobstore.AccumulatingBlobStoreMetrics;
import org.globalart.astrum.blobstore.BlobStoreMetricsNotAvailableException;
import org.globalart.astrum.blobstore.UnavailableBlobStoreMetrics;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreMetrics;
import org.globalart.astrum.blobstore.api.OperationMetrics;
import org.globalart.astrum.blobstore.api.OperationType;
import org.globalart.astrum.blobstore.api.metrics.BlobStoreMetricsPropertiesReader;
import org.globalart.astrum.common.property.ImplicitSourcePropertiesFile;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;

import com.google.common.annotations.VisibleForTesting;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import static java.lang.Long.parseLong;
import static java.util.concurrent.TimeUnit.MILLISECONDS;
import static java.util.stream.Stream.iterate;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.NEW;
import static org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport.State.STARTED;

/**
 * @deprecated legacy method for metrics stored in the blob store, only used for migrating data to the db
 */
@Deprecated
public abstract class BlobStoreMetricsPropertiesReaderSupport<B extends BlobStore, T extends ImplicitSourcePropertiesFile>
    extends StateGuardLifecycleSupport
    implements BlobStoreMetricsPropertiesReader<B>
{
  private static final int METRICS_LOADING_DELAY_MILLIS = 200;

  public static final int MAXIMUM_TRIES = 3;

  public static final String METRICS_FILENAME = "metrics.properties";

  @VisibleForTesting
  public static final String TOTAL_SIZE_PROP_NAME = "totalSize";

  @VisibleForTesting
  public static final String BLOB_COUNT_PROP_NAME = "blobCount";

  protected B blobStore;

  private Map<OperationType, OperationMetrics> operationMetrics;

  @Override
  public final void init(final B blobStore) throws Exception {
    initMetrics(blobStore);
    this.start();
  }

  @Guarded(by = NEW)
  private void initMetrics(final B blobStore) throws Exception {
    checkState(this.blobStore == null, "Do not initialize twice");

    this.blobStore = checkNotNull(blobStore);
    doInit(blobStore);
  }

  /**
   * Called during {@link init} for subclasses to derive necessary values from the configured blobstore
   */
  protected abstract void doInit(B blobstore) throws Exception;

  @Override
  protected void doStart() throws Exception {
    operationMetrics = new EnumMap<>(OperationType.class);
    for (OperationType type : OperationType.values()) {
      operationMetrics.put(type, new OperationMetrics());
    }
  }

  @Override
  protected void doStop() throws Exception {
    blobStore = null;

    operationMetrics.clear();
  }

  protected abstract AccumulatingBlobStoreMetrics getAccumulatingBlobStoreMetrics() throws BlobStoreMetricsNotAvailableException;

  protected abstract Stream<T> backingFiles() throws BlobStoreMetricsNotAvailableException;

  protected BlobStoreMetrics getCombinedMetrics(
      final Stream<T> blobStoreMetricsFiles) throws BlobStoreMetricsNotAvailableException
  {
    AccumulatingBlobStoreMetrics blobStoreMetrics = getAccumulatingBlobStoreMetrics();
    blobStoreMetricsFiles.forEach(metricsFile -> {
      iterate(1, i -> i + 1)
          .limit(MAXIMUM_TRIES)
          .forEach(currentTry -> {
            try {
              metricsFile.load();
            }
            catch (IOException e) {
              log.debug("Unable to load properties file {}. Try number {} of {}.", metricsFile, currentTry,
                  MAXIMUM_TRIES, e);
              if (currentTry >= MAXIMUM_TRIES) {
                throw new RuntimeException("Failed to load blob store metrics from " + metricsFile, e);
              }
              try {
                MILLISECONDS.sleep(METRICS_LOADING_DELAY_MILLIS);
              }
              catch (InterruptedException e1) {
                log.warn("Interrupted", e1);
                Thread.currentThread().interrupt();
              }
            }
          });

      blobStoreMetrics.addBlobCount(parseLong(metricsFile.getProperty(BLOB_COUNT_PROP_NAME, "0")));
      blobStoreMetrics.addTotalSize(parseLong(metricsFile.getProperty(TOTAL_SIZE_PROP_NAME, "0")));
    });
    return blobStoreMetrics;
  }

  @Override
  @Guarded(by = STARTED)
  public BlobStoreMetrics getMetrics() {
    try {
      return getCombinedMetrics(backingFiles());
    }
    catch (BlobStoreMetricsNotAvailableException e) {
      log.error("Blob store metrics cannot be accessed", e);
      return UnavailableBlobStoreMetrics.getInstance();
    }
  }

  @Override
  public Map<OperationType, OperationMetrics> getOperationMetrics() {
    return operationMetrics;
  }

}
