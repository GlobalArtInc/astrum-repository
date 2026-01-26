
package org.globalart.astrum.blobstore.s3.internal;

import java.util.Objects;
import java.util.stream.Stream;

import org.globalart.astrum.blobstore.AccumulatingBlobStoreMetrics;
import org.globalart.astrum.blobstore.BlobStoreMetricsNotAvailableException;
import org.globalart.astrum.blobstore.metrics.BlobStoreMetricsPropertiesReaderSupport;

import software.amazon.awssdk.core.exception.SdkClientException;
import com.google.common.collect.ImmutableMap;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * A S3 specific {@link BlobStoreMetricsPropertiesReaderSupport} implementation that retains blobstore metrics in
 * memory,
 * periodically writing them out to S3.
 *
 * @since 3.6.1
 * @deprecated legacy method for metrics stored in the blob store
 */
@Deprecated
@Component
@Qualifier(S3BlobStore.TYPE)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class S3BlobStoreMetricsPropertiesReader
    extends BlobStoreMetricsPropertiesReaderSupport<S3BlobStore, S3PropertiesFile>
{
  static final ImmutableMap<String, Long> AVAILABLE_SPACE_BY_FILE_STORE = ImmutableMap.of("s3", Long.MAX_VALUE);

  private String bucket;

  private String bucketPrefix;

  private EncryptingS3Client s3;

  @Override
  protected AccumulatingBlobStoreMetrics getAccumulatingBlobStoreMetrics() {
    return new AccumulatingBlobStoreMetrics(0, 0, AVAILABLE_SPACE_BY_FILE_STORE, true);
  }

  @Override
  protected Stream<S3PropertiesFile> backingFiles() throws BlobStoreMetricsNotAvailableException {
    try {
      if (s3 == null) {
        return Stream.empty();
      }
      else {
        return s3.listObjectsV2(bucket, bucketPrefix)
            .contents()
            .stream()
            .filter(Objects::nonNull)
            .filter(s3Object -> s3Object.key().endsWith(METRICS_FILENAME))
            .map(s3Object -> new S3PropertiesFile(s3, bucket, s3Object.key()));
      }
    }
    catch (SdkClientException e) {
      throw new BlobStoreMetricsNotAvailableException(e);
    }
  }

  @Override
  protected void doInit(final S3BlobStore blobstore) throws Exception {
    this.bucket = checkNotNull(blobstore.getConfiguredBucket());
    this.bucketPrefix = checkNotNull(blobstore.getBucketPrefix());
    this.s3 = checkNotNull(blobstore.getS3());
  }
}
