
package org.globalart.astrum.repository.content.store;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.blobstore.api.BlobRef;
import org.globalart.astrum.blobstore.api.HeavyBlobRef;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;

/**
 * Tests for {@link AssetBlobData#blobRef()} conditional logic that returns
 * {@link HeavyBlobRef} when sha1 checksum and blobCreated are both present.
 */
public class AssetBlobDataTest
{
  private static final OffsetDateTime TIME = OffsetDateTime.of(2024, 1, 15, 10, 30, 0, 0, ZoneOffset.UTC);

  private AssetBlobData underTest;

  @Before
  public void setUp() {
    underTest = new AssetBlobData();
  }

  @Test
  public void testBlobRefWithSha1ReturnsHeavyBlobRef() {
    BlobRef blobRef = new BlobRef("node", "store", "blob-id");
    Map<String, String> checksums = new HashMap<>();
    checksums.put("sha1", "abc123sha1hash");

    underTest.setBlobRef(blobRef);
    underTest.setChecksums(checksums);
    underTest.setBlobCreated(TIME);
    underTest.setBlobSize(512L);

    assertThat(underTest.blobRef(), instanceOf(HeavyBlobRef.class));
  }

  @Test
  public void testBlobRefWithoutSha1ReturnsRegularBlobRef() {
    BlobRef blobRef = new BlobRef("node", "store", "blob-id");
    Map<String, String> checksums = new HashMap<>();
    checksums.put("MD5", "abc123md5hash");

    underTest.setBlobRef(blobRef);
    underTest.setChecksums(checksums);
    underTest.setBlobCreated(TIME);

    assertThat(underTest.blobRef(), not(instanceOf(HeavyBlobRef.class)));
  }

  @Test
  public void testBlobRefWithNullChecksumsReturnsRegularBlobRef() {
    BlobRef blobRef = new BlobRef("node", "store", "blob-id");
    underTest.setBlobRef(blobRef);

    assertThat(underTest.blobRef(), not(instanceOf(HeavyBlobRef.class)));
    assertThat(underTest.blobRef(), is(blobRef));
  }

  @Test
  public void testBlobRefWithSha1ButNullBlobCreatedReturnsRegularBlobRef() {
    BlobRef blobRef = new BlobRef("node", "store", "blob-id");
    Map<String, String> checksums = new HashMap<>();
    checksums.put("sha1", "abc123sha1hash");

    underTest.setBlobRef(blobRef);
    underTest.setChecksums(checksums);

    assertThat(underTest.blobRef(), not(instanceOf(HeavyBlobRef.class)));
  }
}
