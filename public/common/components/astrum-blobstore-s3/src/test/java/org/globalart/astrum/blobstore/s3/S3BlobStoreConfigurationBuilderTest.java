
package org.globalart.astrum.blobstore.s3;

import org.globalart.astrum.blobstore.MockBlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.core.Is.is;
import static org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper.CONFIG_KEY;
import static org.globalart.astrum.blobstore.s3.internal.S3BlobStore.PRE_SIGNED_URL_ENABLED;

public class S3BlobStoreConfigurationBuilderTest
{
  @Test
  public void preSignedUrlShouldBeTrue() {
    final MockBlobStoreConfiguration configuration = new MockBlobStoreConfiguration("test-blobstore", "s3");
    final BlobStoreConfiguration config =
        S3BlobStoreConfigurationBuilder.builder(configuration, "test")
            .bucket("test-bucket")
            .region("test-region")
            .preSignedUrlEnabled(true)
            .build();

    assertThat(config.attributes(CONFIG_KEY).get(PRE_SIGNED_URL_ENABLED), is(true));
  }

  @Test
  public void preSignedUrlShouldBeFalse() {
    final MockBlobStoreConfiguration configuration = new MockBlobStoreConfiguration("test-blobstore", "s3");
    final BlobStoreConfiguration config =
        S3BlobStoreConfigurationBuilder.builder(configuration, "test")
            .bucket("test-bucket")
            .region("test-region")
            .preSignedUrlEnabled(false)
            .build();

    assertThat(config.attributes(CONFIG_KEY).get(PRE_SIGNED_URL_ENABLED), is(false));
  }

  @Test
  public void preSignedUrlShouldBeNull() {
    final MockBlobStoreConfiguration configuration = new MockBlobStoreConfiguration("test-blobstore", "s3");
    final BlobStoreConfiguration config =
        S3BlobStoreConfigurationBuilder.builder(configuration, "test")
            .bucket("test-bucket")
            .region("test-region")
            .build();

    assertThat(config.attributes(CONFIG_KEY).get(PRE_SIGNED_URL_ENABLED), is(nullValue()));
  }
}
