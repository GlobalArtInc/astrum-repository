
package org.globalart.astrum.repository.content.store;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.Assert.assertThrows;

/**
 * Tests for {@link AssetBlobCreatedContinuationToken}.
 */
public class AssetBlobCreatedContinuationTokenTest
{
  @Test
  public void testEncodeDecode() {
    long blobCreatedMillis = System.currentTimeMillis();
    int assetId = 42;

    AssetBlobCreatedContinuationToken token = new AssetBlobCreatedContinuationToken(blobCreatedMillis, assetId);
    String encoded = token.encode();

    assertThat(encoded, notNullValue());

    AssetBlobCreatedContinuationToken decoded = AssetBlobCreatedContinuationToken.decode(encoded);

    assertThat(decoded, notNullValue());
    assertThat(decoded.getBlobCreatedMillis(), is(blobCreatedMillis));
    assertThat(decoded.getAssetId(), is(assetId));
  }

  @Test
  public void testGetBlobCreated() {
    long blobCreatedMillis = 1682460420786L; // Apr 25, 2023
    int assetId = 123;

    AssetBlobCreatedContinuationToken token = new AssetBlobCreatedContinuationToken(blobCreatedMillis, assetId);
    OffsetDateTime blobCreated = token.getBlobCreated();

    assertThat(blobCreated, notNullValue());
    assertThat(blobCreated.toInstant().toEpochMilli(), is(blobCreatedMillis));
    assertThat(blobCreated.getOffset(), is(ZoneOffset.UTC));
  }

  @Test
  public void testDecodeNull() {
    AssetBlobCreatedContinuationToken token = AssetBlobCreatedContinuationToken.decode(null);
    assertThat(token, nullValue());
  }

  @Test
  public void testDecodeEmpty() {
    AssetBlobCreatedContinuationToken token = AssetBlobCreatedContinuationToken.decode("");
    assertThat(token, nullValue());
  }

  @Test
  public void testDecodeInvalidFormat() {
    assertThrows(IllegalArgumentException.class,
        () -> AssetBlobCreatedContinuationToken.decode("invalid-token"));
  }

  @Test
  public void testDecodeInvalidBase64() {
    assertThrows(IllegalArgumentException.class,
        () -> AssetBlobCreatedContinuationToken.decode("!!!not-base64!!!"));
  }

  @Test
  public void testDecodeMissingColon() {
    // Valid base64 but no colon separator
    String token = java.util.Base64.getUrlEncoder().withoutPadding().encodeToString("12345".getBytes());
    assertThrows(IllegalArgumentException.class,
        () -> AssetBlobCreatedContinuationToken.decode(token));
  }

  @Test
  public void testDecodeNonNumericTimestamp() {
    String token = java.util.Base64.getUrlEncoder()
        .withoutPadding()
        .encodeToString("notanumber:123".getBytes());
    assertThrows(IllegalArgumentException.class,
        () -> AssetBlobCreatedContinuationToken.decode(token));
  }

  @Test
  public void testDecodeNonNumericAssetId() {
    String token = java.util.Base64.getUrlEncoder()
        .withoutPadding()
        .encodeToString("1234567890:notanumber".getBytes());
    assertThrows(IllegalArgumentException.class,
        () -> AssetBlobCreatedContinuationToken.decode(token));
  }

  @Test
  public void testEquality() {
    AssetBlobCreatedContinuationToken token1 = new AssetBlobCreatedContinuationToken(1000L, 42);
    AssetBlobCreatedContinuationToken token2 = new AssetBlobCreatedContinuationToken(1000L, 42);
    AssetBlobCreatedContinuationToken token3 = new AssetBlobCreatedContinuationToken(2000L, 42);
    AssetBlobCreatedContinuationToken token4 = new AssetBlobCreatedContinuationToken(1000L, 43);

    assertThat(token1, equalTo(token2));
    assertThat(token1.equals(token3), is(false));
    assertThat(token1.equals(token4), is(false));
  }

  @Test
  public void testHashCode() {
    AssetBlobCreatedContinuationToken token1 = new AssetBlobCreatedContinuationToken(1000L, 42);
    AssetBlobCreatedContinuationToken token2 = new AssetBlobCreatedContinuationToken(1000L, 42);

    assertThat(token1.hashCode(), is(token2.hashCode()));
  }

  @Test
  public void testToString() {
    AssetBlobCreatedContinuationToken token = new AssetBlobCreatedContinuationToken(1682460420786L, 123);
    String str = token.toString();

    assertThat(str, notNullValue());
    assertThat(str.contains("1682460420786"), is(true));
    assertThat(str.contains("123"), is(true));
  }

  @Test
  public void testRoundTripWithVariousTimestamps() {
    // Test with epoch 0
    testRoundTrip(0L, 1);

    // Test with recent timestamp
    testRoundTrip(System.currentTimeMillis(), 100);

    // Test with maximum long value
    testRoundTrip(Long.MAX_VALUE, Integer.MAX_VALUE);
  }

  private void testRoundTrip(final long blobCreatedMillis, final int assetId) {
    AssetBlobCreatedContinuationToken original = new AssetBlobCreatedContinuationToken(blobCreatedMillis, assetId);
    String encoded = original.encode();
    AssetBlobCreatedContinuationToken decoded = AssetBlobCreatedContinuationToken.decode(encoded);

    assertThat(decoded.getBlobCreatedMillis(), is(blobCreatedMillis));
    assertThat(decoded.getAssetId(), is(assetId));
    assertThat(decoded, equalTo(original));
  }
}
