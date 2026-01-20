
package org.globalart.astrum.api.rest.common.blobstore.s3;

import java.util.List;

import org.globalart.astrum.api.rest.common.blobstore.model.BlobStoreApiSoftQuota;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiAdvancedBucketConnection;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiBucket;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiBucketConfiguration;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiBucketSecurity;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiEncryption;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiFailoverBucket;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiModel;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;

import org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationBuilder;
import org.globalart.astrum.common.app.ApplicationVersion;

import com.google.common.base.Preconditions;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Transforms a {@link S3BlobStoreApiModel} to an {@link BlobStoreConfiguration}.
 *
 * @since 3.20
 */
public final class S3BlobStoreApiModelMapper
{
  public static final String PRO = "PRO";

  public static BlobStoreConfiguration map(
      final BlobStoreConfiguration blobStoreConfiguration,
      final S3BlobStoreApiModel request,
      final ApplicationVersion applicationVersion)
  {
    checkNotNull(blobStoreConfiguration);
    checkNotNull(request);
    final S3BlobStoreApiBucketConfiguration bucketConfiguration = checkNotNull(request.getBucketConfiguration());
    S3BlobStoreApiBucket bucket =
        Preconditions.checkNotNull(bucketConfiguration.getBucket(), "Missing bucket configuration");

    S3BlobStoreConfigurationBuilder builder =
        S3BlobStoreConfigurationBuilder.builder(blobStoreConfiguration, request.getName())
            .bucket(bucket.getName())
            .region(bucket.getRegion())
            .prefix(bucket.getPrefix())
            .preSignedUrlEnabled(bucketConfiguration.getPreSignedUrlEnabled());

    S3BlobStoreApiBucketSecurity bucketSecurity = bucketConfiguration.getBucketSecurity();
    if (bucketSecurity != null) {
      builder.accessKey(bucketSecurity.getAccessKeyId());
      builder.accessSecret(bucketSecurity.getSecretAccessKey());
      builder.assumeRole(bucketSecurity.getRole());
      builder.sessionTokenKey(bucketSecurity.getSessionToken());
    }

    S3BlobStoreApiEncryption encryption = bucketConfiguration.getEncryption();
    if (encryption != null) {
      builder.encryptionKey(encryption.getEncryptionKey());
      builder.encryptionType(encryption.getEncryptionType());
    }

    S3BlobStoreApiAdvancedBucketConnection advanced = bucketConfiguration.getAdvancedBucketConnection();
    if (advanced != null) {
      builder.endpoint(advanced.getEndpoint());
      builder.signerType(advanced.getSignerType());
      builder.maxConnectionPool(advanced.getMaxConnectionPoolSize());
      builder.forcePathStyle(advanced.getForcePathStyle());
    }

    List<S3BlobStoreApiFailoverBucket> failoverBuckets = bucketConfiguration.getFailoverBuckets();
    if (failoverBuckets != null) {
      failoverBuckets.forEach(failover -> builder.failover(failover.getRegion(), failover.getBucketName()));
    }

    BlobStoreApiSoftQuota softQuota = request.getSoftQuota();
    if (softQuota != null) {
      builder.quotaConfig(softQuota.getType(), checkNotNull(softQuota.getLimit(), "Missing quota limit"));
    }

    return builder.build();
  }
}
