
package org.globalart.astrum.repository.conda;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.cache.CacheControllerHolder.CacheType;

import static org.globalart.astrum.repository.cache.CacheControllerHolder.CONTENT;
import static org.globalart.astrum.repository.cache.CacheControllerHolder.METADATA;

public enum AssetKind
{
  CHANNEL_INDEX_HTML(METADATA, "index.html", false),
  CHANNEL_DATA_JSON(METADATA, "channeldata.json", false),
  CHANNEL_RSS_XML(METADATA, "rss.xml", false),
  ARCH_INDEX_HTML(METADATA, "index.html", true),
  ARCH_REPODATA_JSON(METADATA, "repodata.json", true),
  ARCH_REPODATA_JSON_BZ2(METADATA, "repodata.json.bz2", true),
  ARCH_REPODATA_JSON_ZST(METADATA, "repodata.json.zst", true),
  ARCH_CURRENT_REPODATA_JSON(METADATA, "current_repodata.json", true),
  ARCH_REPODATA2_JSON(METADATA, "repodata2.json", true),
  ARCH_REPODATA_SHARDS(METADATA, "repodata_shards.msgpack.zst", true),
  ARCH_SHARD(CONTENT, null, true),
  ARCH_TAR_PACKAGE(CONTENT, null, true),
  ARCH_CONDA_PACKAGE(CONTENT, null, true);

  private final CacheType cacheType;

  private final String fileName;

  private final boolean subdirLevel;

  AssetKind(final CacheType cacheType, @Nullable final String fileName, final boolean subdirLevel) {
    this.cacheType = cacheType;
    this.fileName = fileName;
    this.subdirLevel = subdirLevel;
  }

  @Nonnull
  public CacheType getCacheType() {
    return cacheType;
  }

  @Nullable
  public String getFileName() {
    return fileName;
  }

  public boolean isSubdirLevel() {
    return subdirLevel;
  }

  public boolean isPackage() {
    return this == ARCH_TAR_PACKAGE || this == ARCH_CONDA_PACKAGE;
  }

  public static boolean isPackage(@Nullable final String kind) {
    return ARCH_TAR_PACKAGE.name().equals(kind) || ARCH_CONDA_PACKAGE.name().equals(kind);
  }

  public static boolean isRepodata(@Nullable final String kind) {
    return ARCH_REPODATA_JSON.name().equals(kind) || ARCH_REPODATA_JSON_ZST.name().equals(kind);
  }
}
