package org.globalart.astrum.repository.yum;

import java.io.IOException;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.repository.yum.internal.rpm.RpmPackage;

@Facet.Exposed
public interface YumContentFacet
    extends ContentFacet
{
  String ATTRIBUTES_KEY = YumFormat.NAME;

  Optional<FluentAsset> getAsset(String path);

  Optional<Content> get(String path);

  TempBlob ingest(Payload payload);

  Content putRpm(String path, TempBlob blob, RpmPackage rpmPackage);

  Content putOther(String path, TempBlob blob);

  Content putProxied(String path, AssetKind assetKind, Content content) throws IOException;

  Content putMetadata(String path, AssetKind assetKind, Path file, String contentType, Map<String, Object> attributes)
      throws IOException;

  Content putMetadata(String path, AssetKind assetKind, byte[] bytes, String contentType, Map<String, Object> attributes);

  boolean delete(String path);

  Iterable<FluentAsset> assetsOfKind(AssetKind assetKind);

  List<FluentAsset> assetsUnder(String directory, AssetKind assetKind);
}
