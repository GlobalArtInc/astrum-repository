
package org.globalart.astrum.repository.alpine;

import java.io.IOException;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.alpine.internal.apk.ApkPackage;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

@Facet.Exposed
public interface AlpineContentFacet
    extends ContentFacet
{
  String ATTRIBUTES_KEY = AlpineFormat.NAME;

  Optional<FluentAsset> getAsset(String path);

  Optional<Content> get(String path);

  TempBlob ingest(Payload payload);

  Content putPackage(String path, TempBlob blob, ApkPackage apkPackage);

  Content putProxied(String path, AssetKind assetKind, Content content) throws IOException;

  Content putIndex(String path, byte[] index, Map<String, Object> attributes);

  boolean delete(String path);

  List<FluentAsset> packagesIn(String directory);
}
