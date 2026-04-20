
package org.globalart.astrum.repository.composer;

import java.io.IOException;
import java.io.InputStream;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.cache.CacheInfo;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

/**
 * Content facet used for getting assets from storage and putting assets into storage for a Composer-format repository.
 */
@Facet.Exposed
public interface ComposerContentFacet
    extends ContentFacet
{

  Optional<FluentAsset> getAsset(String path);

  Optional<Content> get(String path);

  Content put(String path, Payload payload, AssetKind assetKind) throws IOException;

  FluentAsset put(String path, Payload payload, String sourceType, String sourceUrl, String sourceReference) throws IOException;

  TempBlob getTempBlob(Payload payload);

  TempBlob getTempBlob(InputStream in, @Nullable String contentType);

  void setCacheInfo(String path, Content content, CacheInfo cacheInfo) throws IOException;
}
