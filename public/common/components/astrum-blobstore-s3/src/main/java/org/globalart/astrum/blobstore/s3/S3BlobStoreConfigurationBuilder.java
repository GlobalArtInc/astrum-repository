
package org.globalart.astrum.blobstore.s3;

import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Optional;
import java.util.function.Consumer;
import java.util.function.Supplier;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.BlobStoreConfigurationBuilder;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.s3.internal.S3BlobStore;
import org.globalart.astrum.common.collect.NestedAttributesMap;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Optional.ofNullable;
import static org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper.BUCKET_KEY;
import static org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper.BUCKET_PREFIX_KEY;
import static org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper.CONFIG_KEY;
import static org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper.FAILOVER_BUCKETS_KEY;
import static org.globalart.astrum.blobstore.s3.internal.S3BlobStore.*;

/**
 * Builder for S3 BlobStoreConfiguration objects.
 *
 * @since 3.37
 */
public class S3BlobStoreConfigurationBuilder
    extends BlobStoreConfigurationBuilder
{
  private String bucket;

  private String region;

  private Optional<String> prefix = Optional.empty();

  private Optional<String> accessKey = Optional.empty();

  private Optional<String> accessSecret = Optional.empty();

  private Optional<String> assumeRole = Optional.empty();

  private Optional<String> sessionTokenKey = Optional.empty();

  private Optional<String> encryptionKey = Optional.empty();

  private Optional<String> encryptionType = Optional.empty();

  private Optional<String> endpoint = Optional.empty();

  private Optional<String> signerType = Optional.empty();

  private Optional<String> maxConnectionPool = Optional.empty();

  private Optional<Boolean> forcePathStyle = Optional.empty();

  private Boolean preSignedUrlEnabled;

  // Uses a linkedhashmap to maintain order

  private Map<String, String> failover = new LinkedHashMap<>();

  private S3BlobStoreConfigurationBuilder(final Supplier<BlobStoreConfiguration> configuration, final String name) {
    super(name, configuration);
    super.type(S3BlobStore.TYPE);
  }

  /**
   * Set the S3 bucket name.
   */
  public S3BlobStoreConfigurationBuilder bucket(final String bucketName) {
    this.bucket = checkNotNull(bucketName, "Missing bucket name");
    return this;
  }

  /**
   * Set the S3 region.
   */
  public S3BlobStoreConfigurationBuilder region(final String region) {
    this.region = checkNotNull(region, "Missing region");
    return this;
  }

  /**
   * Set a prefix to use when storing blobs. This can allow multiple blobstores within the same bucket.
   */
  public S3BlobStoreConfigurationBuilder prefix(@Nullable final String prefix) {
    this.prefix = ofNullable(prefix);
    return this;
  }

  /**
   * Indicate whether pre signed url is enabled or not
   */
  public S3BlobStoreConfigurationBuilder preSignedUrlEnabled(final Boolean preSignedUrlEnabled) {
    this.preSignedUrlEnabled = preSignedUrlEnabled;
    return this;
  }

  public S3BlobStoreConfigurationBuilder failover(final String region, final String bucketName) {
    this.failover.put(region, bucketName);
    return this;
  }

  /**
   * Set the authentication access key.
   */
  public S3BlobStoreConfigurationBuilder accessKey(@Nullable final String accessKey) {
    this.accessKey = ofNullable(accessKey);
    return this;
  }

  /**
   * Set the authentication access secret.
   */
  public S3BlobStoreConfigurationBuilder accessSecret(@Nullable final String accessSecret) {
    this.accessSecret = ofNullable(accessSecret);
    return this;
  }

  /**
   * Set the AWS role to assume.
   */
  public S3BlobStoreConfigurationBuilder assumeRole(@Nullable final String assumeRole) {
    this.assumeRole = ofNullable(assumeRole);
    return this;
  }

  /**
   * Set the session token key.
   */
  public S3BlobStoreConfigurationBuilder sessionTokenKey(@Nullable final String sessionTokenKey) {
    this.sessionTokenKey = ofNullable(sessionTokenKey);
    return this;
  }

  /**
   * Set the encryption key.
   */
  public S3BlobStoreConfigurationBuilder encryptionKey(@Nullable final String encryptionKey) {
    this.encryptionKey = ofNullable(encryptionKey);
    return this;
  }

  /**
   * Set the encryption type.
   */
  public S3BlobStoreConfigurationBuilder encryptionType(@Nullable final String encryptionType) {
    this.encryptionType = ofNullable(encryptionType);
    return this;
  }

  /**
   * Set the endpoint for S3. This overrides the the S3 URL.
   */
  public S3BlobStoreConfigurationBuilder endpoint(@Nullable final String endpoint) {
    this.endpoint = ofNullable(endpoint);
    return this;
  }

  public S3BlobStoreConfigurationBuilder signerType(@Nullable final String signerType) {
    this.signerType = ofNullable(signerType);
    return this;
  }

  /**
   * Set the maximum number of threads used by the connection pool to S3.
   */
  public S3BlobStoreConfigurationBuilder maxConnectionPool(@Nullable final String maxConnectionPool) {
    this.maxConnectionPool = ofNullable(maxConnectionPool);
    return this;
  }

  /**
   * Set the maximum number of threads used by the connection pool to S3.
   */
  public S3BlobStoreConfigurationBuilder maxConnectionPool(@Nullable final Integer maxConnectionPool) {
    this.maxConnectionPool = ofNullable(maxConnectionPool)
        .map(String::valueOf);
    return this;
  }

  /**
   * Force path-style URLs, this is deprecated with AWS but fake S3 appliances may need it.
   */
  public S3BlobStoreConfigurationBuilder forcePathStyle(@Nullable final Boolean forcePathStyle) {
    this.forcePathStyle = ofNullable(forcePathStyle);
    return this;
  }

  /**
   * Force path-style URLs, this is deprecated with AWS but fake S3 appliances may need it.
   */
  public S3BlobStoreConfigurationBuilder forcePathStyle(@Nullable final String forcePathStyle) {
    if (forcePathStyle == null) {
      this.forcePathStyle = Optional.empty();
    }
    else {
      this.forcePathStyle = ofNullable(Boolean.valueOf(forcePathStyle));
    }
    return this;
  }

  /**
   * Sets the type of blob store.
   */
  @Override
  public BlobStoreConfigurationBuilder type(final String type) {
    throw new IllegalStateException("The type cannot be changed.");
  }

  @Override
  public BlobStoreConfiguration build() {
    BlobStoreConfiguration configuration = super.build();

    NestedAttributesMap s3 = configuration.attributes(CONFIG_KEY);

    s3.set(BUCKET_KEY, checkNotNull(bucket, "Missing bucket name"));
    s3.set(REGION_KEY, checkNotNull(region, "Missing region"));

    prefix.ifPresent(set(s3, BUCKET_PREFIX_KEY));

    ofNullable(preSignedUrlEnabled).ifPresent(set(s3, PRE_SIGNED_URL_ENABLED));

    // Authentication
    accessKey.ifPresent(set(s3, ACCESS_KEY_ID_KEY));
    accessSecret.ifPresent(set(s3, SECRET_ACCESS_KEY_KEY));
    assumeRole.ifPresent(set(s3, ASSUME_ROLE_KEY));
    sessionTokenKey.ifPresent(set(s3, SESSION_TOKEN_KEY));

    // encryption
    encryptionKey.ifPresent(set(s3, ENCRYPTION_KEY));
    encryptionType.ifPresent(set(s3, ENCRYPTION_TYPE));

    // advanced
    endpoint.ifPresent(set(s3, ENDPOINT_KEY));
    signerType.ifPresent(set(s3, SIGNERTYPE_KEY));
    maxConnectionPool.ifPresent(set(s3, MAX_CONNECTION_POOL_KEY));

    // failover
    if (!failover.isEmpty()) {
      s3.set(FAILOVER_BUCKETS_KEY, failover);
    }

    // only set if true
    forcePathStyle.filter(b -> b)
        .map(String::valueOf)
        .ifPresent(set(s3, FORCE_PATH_STYLE_KEY));

    return configuration;
  }

  private static Consumer<Object> set(final NestedAttributesMap attributes, final String key) {
    return value -> attributes.set(key, value);
  }

  public static S3BlobStoreConfigurationBuilder builder(final BlobStoreConfiguration configuration, final String name) {
    return new S3BlobStoreConfigurationBuilder(() -> configuration, name);
  }

  public static S3BlobStoreConfigurationBuilder builder(
      final Supplier<BlobStoreConfiguration> configuration,
      final String name)
  {
    return new S3BlobStoreConfigurationBuilder(configuration, name);
  }
}
