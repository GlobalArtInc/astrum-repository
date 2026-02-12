
package org.globalart.astrum.repository.content.store;

import java.time.OffsetDateTime;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

/**
 * Tests for {@link AssetData}, specifically focusing on continuation token generation.
 */
public class AssetDataTest
{
  private AssetData assetData;

  private AssetBlobData assetBlobData;

  @Before
  public void setup() {
    assetData = new AssetData();
    assetData.setAssetId(42);
    assetData.setPath("/test/path");
    assetData.setKind("test");

    assetBlobData = new AssetBlobData();
    assetBlobData.setAssetBlobId(100);
    assetBlobData.setBlobCreated(OffsetDateTime.now());
  }

  @Test
  public void testNextContinuationToken_nonEagerQuery_returnsSimpleAssetId() {
    // Non-eager query (default): should return simple asset ID token
    assetData.setFromEagerQuery(false);
    assetData.setAssetBlob(assetBlobData);

    String token = assetData.nextContinuationToken();

    assertThat(token, is("42"));
  }

  @Test
  public void testNextContinuationToken_eagerQuery_withBlob_returnsCompositeToken() {
    // Eager query with blob: should return composite token
    assetData.setFromEagerQuery(true);
    assetData.setAssetBlob(assetBlobData);

    String token = assetData.nextContinuationToken();

    // Verify it's a composite token by decoding it
    AssetBlobCreatedContinuationToken decodedToken = AssetBlobCreatedContinuationToken.decode(token);
    assertThat(decodedToken, notNullValue());
    assertThat(decodedToken.getAssetId(), is(42));
    assertThat(decodedToken.getBlobCreatedMillis(), is(assetBlobData.blobCreated().toInstant().toEpochMilli()));
  }

  @Test
  public void testNextContinuationToken_eagerQuery_withoutBlob_returnsCompositeTokenWithZero() {
    // Eager query without blob: should return composite token with blobCreatedMillis = 0
    assetData.setFromEagerQuery(true);
    assetData.setAssetBlob(null);

    String token = assetData.nextContinuationToken();

    // Should be a composite token with blobCreatedMillis = 0
    AssetBlobCreatedContinuationToken decoded = AssetBlobCreatedContinuationToken.decode(token);
    assertThat(decoded, notNullValue());
    assertThat(decoded.getAssetId(), is(42));
    assertThat(decoded.getBlobCreatedMillis(), is(0L));
  }

  @Test
  public void testNextContinuationToken_defaultFlag_withBlob_returnsSimpleAssetId() {
    // Default flag value (false) with blob loaded: should return simple asset ID
    // This tests that we're no longer using the heuristic of checking if blob is loaded
    assetData.setAssetBlob(assetBlobData);

    String token = assetData.nextContinuationToken();

    assertThat(token, is("42"));
  }

  @Test
  public void testNextContinuationToken_eagerQuery_multipleAssets() {
    // Test multiple assets with different timestamps to ensure unique tokens
    AssetData asset1 = new AssetData();
    asset1.setAssetId(1);
    asset1.setPath("/test/1");
    asset1.setKind("test");
    asset1.setFromEagerQuery(true);

    AssetBlobData blob1 = new AssetBlobData();
    blob1.setAssetBlobId(101);
    blob1.setBlobCreated(OffsetDateTime.now().minusHours(1));
    asset1.setAssetBlob(blob1);

    AssetData asset2 = new AssetData();
    asset2.setAssetId(2);
    asset2.setPath("/test/2");
    asset2.setKind("test");
    asset2.setFromEagerQuery(true);

    AssetBlobData blob2 = new AssetBlobData();
    blob2.setAssetBlobId(102);
    blob2.setBlobCreated(OffsetDateTime.now().minusHours(2));
    asset2.setAssetBlob(blob2);

    String token1 = asset1.nextContinuationToken();
    String token2 = asset2.nextContinuationToken();

    // Tokens should be different due to different timestamps and asset IDs
    assertThat(token1.equals(token2), is(false));

    // Both should be composite tokens
    AssetBlobCreatedContinuationToken decoded1 = AssetBlobCreatedContinuationToken.decode(token1);
    AssetBlobCreatedContinuationToken decoded2 = AssetBlobCreatedContinuationToken.decode(token2);

    assertThat(decoded1.getAssetId(), is(1));
    assertThat(decoded2.getAssetId(), is(2));
  }

  @Test
  public void testNextContinuationToken_eagerQuery_sameTimestamp_differentAssetIds() {
    // Test assets with the same timestamp but different asset IDs
    OffsetDateTime sharedTimestamp = OffsetDateTime.now();

    AssetData asset1 = new AssetData();
    asset1.setAssetId(10);
    asset1.setPath("/test/10");
    asset1.setKind("test");
    asset1.setFromEagerQuery(true);

    AssetBlobData blob1 = new AssetBlobData();
    blob1.setAssetBlobId(201);
    blob1.setBlobCreated(sharedTimestamp);
    asset1.setAssetBlob(blob1);

    AssetData asset2 = new AssetData();
    asset2.setAssetId(20);
    asset2.setPath("/test/20");
    asset2.setKind("test");
    asset2.setFromEagerQuery(true);

    AssetBlobData blob2 = new AssetBlobData();
    blob2.setAssetBlobId(202);
    blob2.setBlobCreated(sharedTimestamp);
    asset2.setAssetBlob(blob2);

    String token1 = asset1.nextContinuationToken();
    String token2 = asset2.nextContinuationToken();

    // Tokens should be different due to different asset IDs (even with same timestamp)
    assertThat(token1.equals(token2), is(false));

    AssetBlobCreatedContinuationToken decoded1 = AssetBlobCreatedContinuationToken.decode(token1);
    AssetBlobCreatedContinuationToken decoded2 = AssetBlobCreatedContinuationToken.decode(token2);

    assertThat(decoded1.getBlobCreatedMillis(), is(decoded2.getBlobCreatedMillis()));
    assertThat(decoded1.getAssetId(), is(10));
    assertThat(decoded2.getAssetId(), is(20));
  }

  @Test
  public void testSetFromEagerQuery_toggleBehavior() {
    // Test that we can toggle the flag and behavior changes accordingly
    assetData.setAssetBlob(assetBlobData);

    // Initially false - should return simple token
    assetData.setFromEagerQuery(false);
    String token1 = assetData.nextContinuationToken();
    assertThat(token1, is("42"));

    // Set to true - should return composite token
    assetData.setFromEagerQuery(true);
    String token2 = assetData.nextContinuationToken();
    AssetBlobCreatedContinuationToken decoded = AssetBlobCreatedContinuationToken.decode(token2);
    assertThat(decoded, notNullValue());
    assertThat(decoded.getAssetId(), is(42));

    // Set back to false - should return simple token again
    assetData.setFromEagerQuery(false);
    String token3 = assetData.nextContinuationToken();
    assertThat(token3, is("42"));
  }

  @Test
  public void testNextContinuationToken_nonEagerQuery_withoutBlob_returnsSimpleAssetId() {
    // Non-eager query without blob: should return simple asset ID
    assetData.setFromEagerQuery(false);
    assetData.setAssetBlob(null);

    String token = assetData.nextContinuationToken();

    assertThat(token, is("42"));
  }

  @Test
  public void testNextContinuationToken_differentAssetIds() {
    // Test that different asset IDs produce different simple tokens
    AssetData asset1 = new AssetData();
    asset1.setAssetId(100);
    asset1.setPath("/test/100");
    asset1.setKind("test");
    asset1.setFromEagerQuery(false);

    AssetData asset2 = new AssetData();
    asset2.setAssetId(200);
    asset2.setPath("/test/200");
    asset2.setKind("test");
    asset2.setFromEagerQuery(false);

    String token1 = asset1.nextContinuationToken();
    String token2 = asset2.nextContinuationToken();

    assertThat(token1, is("100"));
    assertThat(token2, is("200"));
    assertThat(token1.equals(token2), is(false));
  }

  @Test
  public void testNextContinuationToken_consistencyWithinSameQueryType() {
    // Test that calling nextContinuationToken multiple times on the same object
    // returns the same token (idempotent)
    assetData.setFromEagerQuery(true);
    assetData.setAssetBlob(assetBlobData);

    String token1 = assetData.nextContinuationToken();
    String token2 = assetData.nextContinuationToken();

    assertThat(token1, equalTo(token2));
  }
}
