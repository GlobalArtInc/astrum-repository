
package org.globalart.astrum.repository.search.sql.internal;

import java.util.Collections;
import java.util.Date;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.time.DateHelper;
import org.globalart.astrum.repository.cache.CacheAttributeUtils;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.store.InternalIds;
import org.globalart.astrum.repository.search.AssetSearchResult;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.search.index.SearchConstants.CHECKSUM;

public class AssetSearchResultImpl
    implements AssetSearchResult
{
  private final String repository;

  private final String format;

  private final Asset asset;

  private final Optional<AssetBlob> blob;

  public AssetSearchResultImpl(
      final Asset asset,
      final Optional<AssetBlob> blob,
      final String format,
      final String repository)
  {
    this.format = checkNotNull(format);
    this.repository = checkNotNull(repository);
    this.asset = checkNotNull(asset);
    this.blob = checkNotNull(blob);
  }

  public Asset getAsset() {
    return asset;
  }

  @Override
  public Map<String, Object> getAttributes() {
    if (!blob.isPresent()) {
      return Collections.unmodifiableMap(asset.attributes().backing());
    }
    // shallow copy
    Map<String, Object> attributes = new HashMap<>(asset.attributes().backing());
    attributes.put(CHECKSUM, blob.get().checksums());
    return Collections.unmodifiableMap(attributes);
  }

  @Override
  public Date getCreated() {
    return DateHelper.toDate(asset.created());
  }

  @Override
  public String getBlobRef() {
    return blob.map(AssetBlob::blobRef)
        .map(Object::toString)
        .orElse(null);
  }

  @Override
  public Date getBinaryUpdated() {
    // AssetBlob is immutable and a new one is created when the binary changes
    return blob.map(AssetBlob::blobCreated)
        .map(DateHelper::toDate)
        .orElse(null);
  }

  @Override
  public Map<String, String> getChecksum() {
    return blob.map(AssetBlob::checksums)
        .orElseGet(Map::of);
  }

  @Override
  public String getContentType() {
    return blob.map(AssetBlob::contentType)
        .orElse(null);
  }

  @Override
  public Long getFileSize() {
    return blob.map(AssetBlob::blobSize).orElse(null);
  }

  @Override
  public String getFormat() {
    return format;
  }

  @Override
  public String getId() {
    return InternalIds.toExternalId(InternalIds.internalAssetId(asset)).getValue();
  }

  @Override
  public Date getLastDownloaded() {
    return asset.lastDownloaded()
        .map(DateHelper::toDate)
        .orElse(null);
  }

  @Override
  public Date getLastModified() {
    return getBinaryUpdated();
  }

  @Override
  public Date getLastVerified() {
    // Extract lastVerified from cache attributes (only for proxy/group repos)
    return CacheAttributeUtils.extractLastVerifiedAsOptional(asset.attributes().backing())
        .orElse(null);
  }

  @Override
  public String getPath() {
    return asset.path();
  }

  @Override
  public String getRepository() {
    return repository;
  }

  @Override
  public String getUploader() {
    return blob.flatMap(AssetBlob::createdBy).orElse(null);
  }

  @Override
  public String getUploaderIp() {
    return blob.flatMap(AssetBlob::createdByIp).orElse(null);
  }
}
