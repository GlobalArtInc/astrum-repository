package org.globalart.astrum.repository.conda;

import java.io.InputStream;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

@Facet.Exposed
public interface CondaContentFacet
    extends ContentFacet
{
  String INDEX_ATTRIBUTE = "index";

  Optional<FluentAsset> getAsset(String path);

  FluentAsset putMetadata(String path, AssetKind assetKind, TempBlob tempBlob);

  FluentAsset putPackage(
      String path,
      AssetKind assetKind,
      String subdir,
      String name,
      String version,
      TempBlob tempBlob,
      @Nullable Map<String, Object> index);

  List<FluentAsset> packagesIn(String subdirPath);

  boolean delete(String path);

  TempBlob getTempBlob(Payload payload);

  TempBlob getTempBlob(InputStream in, @Nullable String contentType);
}
