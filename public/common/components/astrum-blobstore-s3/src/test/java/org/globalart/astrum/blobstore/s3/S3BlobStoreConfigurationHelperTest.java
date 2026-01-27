
package org.globalart.astrum.blobstore.s3;

import java.util.Collections;
import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.common.collect.NestedAttributesMap;

import org.mockito.MockedStatic;
import org.mockito.Mockito;
import software.amazon.awssdk.regions.Region;
import com.google.common.collect.ImmutableMap;
import org.junit.Test;
import software.amazon.awssdk.regions.providers.DefaultAwsRegionProviderChain;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.mockStatic;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper.CONFIG_KEY;
import static org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper.getConfiguredBucket;
import static org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper.getConfiguredRegion;

public class S3BlobStoreConfigurationHelperTest
{
  @Test
  public void testGetConfiguredBucket() {
    // null doesn't cause failure
    withRegion(null,
        () -> assertThat(getConfiguredBucket(configuration("default", "def-bucket", null)), is("def-bucket")));

    // empty doesn't cause failure
    withRegion(null,
        () -> assertThat(getConfiguredBucket(configuration("default", "def-bucket", Collections.emptyMap())),
            is("def-bucket")));

    BlobStoreConfiguration configuration =
        configuration("default", "def-bucket", ImmutableMap.of("us-east-1", "us-e-bucket"));

    // no region should use primary
    withRegion(null, () -> assertThat(getConfiguredBucket(configuration), is("def-bucket")));

    // Match fail over region
    withRegion(Region.US_EAST_1, () -> assertThat(getConfiguredBucket(configuration), is("us-e-bucket")));

    // Running region doesn't match anything, use primary
    withRegion(Region.AF_SOUTH_1, () -> assertThat(getConfiguredBucket(configuration), is("def-bucket")));
  }

  @Test
  public void testGetConfiguredRegion() {
    // null doesn't cause failure
    withRegion(null,
        () -> assertThat(getConfiguredRegion(configuration("default", "def-bucket", null)), is("default")));

    // empty doesn't cause failure
    withRegion(null,
        () -> assertThat(getConfiguredRegion(configuration("default", "def-bucket", Collections.emptyMap())),
            is("default")));

    BlobStoreConfiguration configuration =
        configuration("default", "def-bucket", ImmutableMap.of("us-east-1", "us-e-bucket"));

    // no region should use primary
    withRegion(null, () -> assertThat(getConfiguredRegion(configuration), is("default")));

    // Match fail over region
    withRegion(Region.US_EAST_1, () -> assertThat(getConfiguredRegion(configuration), is("us-east-1")));

    // Running region doesn't match anything, use primary
    withRegion(Region.AF_SOUTH_1, () -> assertThat(getConfiguredRegion(configuration), is("default")));
  }

  private static void withRegion(final Region region, final Runnable runnable) {
    S3BlobStoreConfigurationHelper.region = null;
    S3BlobStoreConfigurationHelper.regionLoaded = false;

    try (MockedStatic<DefaultAwsRegionProviderChain> regionProviderMock =
        mockStatic(DefaultAwsRegionProviderChain.class)) {
      DefaultAwsRegionProviderChain mockChain = Mockito.mock(DefaultAwsRegionProviderChain.class);

      DefaultAwsRegionProviderChain.Builder mockBuilder = Mockito.mock(DefaultAwsRegionProviderChain.Builder.class);
      when(mockBuilder.build()).thenReturn(mockChain);
      when(mockChain.getRegion()).thenReturn(region);

      regionProviderMock.when(DefaultAwsRegionProviderChain::builder).thenReturn(mockBuilder);

      runnable.run();
    }
  }

  private static BlobStoreConfiguration configuration(
      final String region,
      final String bucketName,
      final Map<String, String> failoverBuckets)
  {
    BlobStoreConfiguration configuration = mock(BlobStoreConfiguration.class);
    NestedAttributesMap attributes = new NestedAttributesMap();
    when(configuration.attributes(CONFIG_KEY)).thenReturn(attributes);

    // use the builder so we get the real code path for creating a config
    S3BlobStoreConfigurationBuilder builder = S3BlobStoreConfigurationBuilder.builder(configuration, "my-blobstore")
        .bucket(bucketName)
        .region(region);

    if (failoverBuckets != null) {
      failoverBuckets.forEach((failoverRegion, failoverBucket) -> builder.failover(failoverRegion, failoverBucket));
    }

    return builder.build();
  }
}
