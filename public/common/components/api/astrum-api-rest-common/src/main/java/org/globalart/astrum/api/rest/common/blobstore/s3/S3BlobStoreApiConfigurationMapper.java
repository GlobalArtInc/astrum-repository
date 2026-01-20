
package org.globalart.astrum.api.rest.common.blobstore.s3;

import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.stream.Collectors;

import org.globalart.astrum.api.rest.common.blobstore.model.BlobStoreApiSoftQuota;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiAdvancedBucketConnection;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiBucket;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiBucketConfiguration;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiBucketSecurity;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiEncryption;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiFailoverBucket;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiModel;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;

import org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper;
import org.globalart.astrum.blobstore.s3.internal.S3BlobStore;
import org.globalart.astrum.common.collect.NestedAttributesMap;

import static java.lang.Long.parseLong;
import static java.util.Objects.nonNull;
import static org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport.LIMIT_KEY;
import static org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport.ROOT_KEY;
import static org.globalart.astrum.blobstore.quota.BlobStoreQuotaSupport.TYPE_KEY;
import static org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper.BUCKET_KEY;
import static org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper.BUCKET_PREFIX_KEY;
import static org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper.CONFIG_KEY;
import static org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper.FAILOVER_BUCKETS_KEY;
import static org.globalart.astrum.blobstore.s3.internal.S3BlobStore.REGION_KEY;

/**
 * Transforms a {@link BlobStoreConfiguration} to an {@link S3BlobStoreApiModel}.
 *
 * @since 3.20
 */
public final class S3BlobStoreApiConfigurationMapper
{
  public static S3BlobStoreApiModel map(final BlobStoreConfiguration configuration) {
    return new S3BlobStoreApiModel(
        configuration.getName(),
        createSoftQuota(configuration),
        createS3BlobStoreBucketConfiguration(configuration));
  }

  private static BlobStoreApiSoftQuota createSoftQuota(final BlobStoreConfiguration configuration) {
    final NestedAttributesMap softQuotaAttributes = configuration.attributes(ROOT_KEY);
    if (!softQuotaAttributes.isEmpty()) {
      final String quotaType = getValue(softQuotaAttributes, TYPE_KEY);
      final String quotaLimit = getValue(softQuotaAttributes, LIMIT_KEY);
      if (nonNull(quotaType) && nonNull(quotaLimit)) {
        final BlobStoreApiSoftQuota blobStoreApiSoftQuota = new BlobStoreApiSoftQuota();
        blobStoreApiSoftQuota.setType(quotaType);
        blobStoreApiSoftQuota.setLimit(parseLong(quotaLimit));
        return blobStoreApiSoftQuota;
      }
    }
    return null;
  }

  private static String getValue(final NestedAttributesMap attributes, final String key) {
    return Objects.toString(attributes.get(key), null);
  }

  private static S3BlobStoreApiBucketConfiguration createS3BlobStoreBucketConfiguration(
      final BlobStoreConfiguration configuration)
  {
    final NestedAttributesMap s3BucketAttributes = configuration.attributes(CONFIG_KEY);
    return new S3BlobStoreApiBucketConfiguration(
        buildS3BlobStoreBucket(s3BucketAttributes),
        buildS3BlobStoreBucketSecurity(s3BucketAttributes),
        buildS3BlobStoreEncryption(s3BucketAttributes),
        buildS3BlobStoreAdvancedBucketConnection(s3BucketAttributes),
        buildS3BlobStoreFailoverBuckets(s3BucketAttributes),
        buildS3BlobStoreActiveRegion(configuration),
        configuration.attributes(CONFIG_KEY).get("preSignedUrlEnabled", Boolean.class));
  }

  private static S3BlobStoreApiBucket buildS3BlobStoreBucket(final NestedAttributesMap attributes) {
    return new S3BlobStoreApiBucket(
        getValue(attributes, REGION_KEY),
        getValue(attributes, BUCKET_KEY),
        getValue(attributes, BUCKET_PREFIX_KEY));
  }

  private static S3BlobStoreApiBucketSecurity buildS3BlobStoreBucketSecurity(final NestedAttributesMap attributes) {
    final String accessKeyId = getValue(attributes, S3BlobStore.ACCESS_KEY_ID_KEY);
    final String roleToAssume = getValue(attributes, S3BlobStore.ASSUME_ROLE_KEY);
    final String sessionToken = getValue(attributes, S3BlobStore.SESSION_TOKEN_KEY);
    if (iamCredentialsProvided(accessKeyId, roleToAssume, sessionToken)) {
      return new S3BlobStoreApiBucketSecurity(accessKeyId, null, roleToAssume, sessionToken);
    }
    return null;
  }

  private static boolean iamCredentialsProvided(
      final String accessKeyId,
      final String roleToAssume,
      final String sessionToken)
  {
    return anyNonNull(accessKeyId, roleToAssume, sessionToken);
  }

  private static S3BlobStoreApiEncryption buildS3BlobStoreEncryption(final NestedAttributesMap s3BucketAttributes) {
    final String encryptionType = getValue(s3BucketAttributes, S3BlobStore.ENCRYPTION_TYPE);
    final String encryptionKey = getValue(s3BucketAttributes, S3BlobStore.ENCRYPTION_KEY);

    if (anyNonNull(encryptionKey, encryptionType)) {
      return new S3BlobStoreApiEncryption(encryptionType, encryptionKey);
    }
    return null;
  }

  private static S3BlobStoreApiAdvancedBucketConnection buildS3BlobStoreAdvancedBucketConnection(
      final NestedAttributesMap attributes)
  {
    final String endpoint = getValue(attributes, S3BlobStore.ENDPOINT_KEY);
    final String signerType = getValue(attributes, S3BlobStore.SIGNERTYPE_KEY);
    final String forcePathStyle = getValue(attributes, S3BlobStore.FORCE_PATH_STYLE_KEY);
    Integer maxConnectionPoolSize =
        Optional.ofNullable(getValue(attributes, S3BlobStore.MAX_CONNECTION_POOL_KEY))
            .filter(val -> !"".equals(val))
            .map(Integer::valueOf)
            .orElse(null);

    if (anyNonNull(endpoint, signerType, forcePathStyle, maxConnectionPoolSize)) {
      return new S3BlobStoreApiAdvancedBucketConnection(endpoint, signerType, Boolean.valueOf(forcePathStyle),
          maxConnectionPoolSize);
    }
    return null;
  }

  private static List<S3BlobStoreApiFailoverBucket> buildS3BlobStoreFailoverBuckets(
      final NestedAttributesMap attributes)
  {
    if (attributes.contains(FAILOVER_BUCKETS_KEY)) {
      return attributes.child(FAILOVER_BUCKETS_KEY)
          .entries()
          .stream()
          .map(entry -> new S3BlobStoreApiFailoverBucket(entry.getKey(), entry.getValue().toString()))
          .collect(Collectors.toList());
    }
    return Collections.emptyList();
  }

  private static String buildS3BlobStoreActiveRegion(final BlobStoreConfiguration configuration) {
    return S3BlobStoreConfigurationHelper.getConfiguredRegion(configuration);
  }

  private static boolean anyNonNull(final Object... objects) {
    for (Object o : objects) {
      if (o != null) {
        return true;
      }
    }
    return false;
  }
}
