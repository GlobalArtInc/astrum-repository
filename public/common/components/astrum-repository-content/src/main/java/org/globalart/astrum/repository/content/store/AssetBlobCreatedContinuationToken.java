
package org.globalart.astrum.repository.content.store;

import java.time.Instant;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.Base64;
import java.util.Objects;
import javax.annotation.Nullable;

/**
 * Composite continuation token that encodes both blob_created timestamp and asset_id for stable pagination
 * when browsing assets ordered by blob_created descending.
 *
 * Format: base64(blobCreatedMillis:assetId)
 */
public class AssetBlobCreatedContinuationToken
{
  private final long blobCreatedMillis;

  private final int assetId;

  public AssetBlobCreatedContinuationToken(final long blobCreatedMillis, final int assetId) {
    this.blobCreatedMillis = blobCreatedMillis;
    this.assetId = assetId;
  }

  public long getBlobCreatedMillis() {
    return blobCreatedMillis;
  }

  public OffsetDateTime getBlobCreated() {
    return OffsetDateTime.ofInstant(Instant.ofEpochMilli(blobCreatedMillis), ZoneOffset.UTC);
  }

  public int getAssetId() {
    return assetId;
  }

  /**
   * Encode the token as a base64 string
   */
  public String encode() {
    String tokenStr = blobCreatedMillis + ":" + assetId;
    return Base64.getUrlEncoder().withoutPadding().encodeToString(tokenStr.getBytes());
  }

  /**
   * Decode a continuation token string into a composite token.
   *
   * @param token the encoded token string
   * @return the decoded token, or null if the token is null
   */
  @Nullable
  public static AssetBlobCreatedContinuationToken decode(@Nullable final String token) {
    if (token == null || token.isEmpty()) {
      return null;
    }

    try {
      String decoded = new String(Base64.getUrlDecoder().decode(token));
      String[] parts = decoded.split(":", 2);
      if (parts.length != 2) {
        throw new IllegalArgumentException("Invalid token format");
      }

      long blobCreatedMillis = Long.parseLong(parts[0]);
      int assetId = Integer.parseInt(parts[1]);

      return new AssetBlobCreatedContinuationToken(blobCreatedMillis, assetId);
    }
    catch (IllegalArgumentException e) {
      throw new IllegalArgumentException("Failed to decode continuation token: " + token, e);
    }
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    AssetBlobCreatedContinuationToken that = (AssetBlobCreatedContinuationToken) o;
    return blobCreatedMillis == that.blobCreatedMillis && assetId == that.assetId;
  }

  @Override
  public int hashCode() {
    return Objects.hash(blobCreatedMillis, assetId);
  }

  @Override
  public String toString() {
    return "AssetBlobCreatedContinuationToken{" +
        "blobCreatedMillis=" + blobCreatedMillis +
        ", assetId=" + assetId +
        '}';
  }
}
