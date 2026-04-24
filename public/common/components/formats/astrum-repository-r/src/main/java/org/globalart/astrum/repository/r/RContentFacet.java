
package org.globalart.astrum.repository.r;

import java.io.IOException;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

@Facet.Exposed
public interface RContentFacet
    extends ContentFacet
{
  String ATTRIBUTES_KEY = RFormat.NAME;

  Optional<FluentAsset> getAsset(String path);

  Optional<Content> get(String path);

  TempBlob ingest(Payload payload);

  FluentAsset putPackage(String path, TempBlob blob, Map<String, String> description);

  Content putProxied(String path, AssetKind assetKind, Content content) throws IOException;

  Content putMetadata(String path, byte[] bytes, String contentType, Map<String, Object> attributes);

  boolean delete(String path);

  List<FluentAsset> archivesIn(String directory);
}
