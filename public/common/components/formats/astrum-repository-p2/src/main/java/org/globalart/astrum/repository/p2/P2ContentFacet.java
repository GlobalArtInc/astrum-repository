
package org.globalart.astrum.repository.p2;

import java.io.IOException;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.p2.internal.AssetKind;
import org.globalart.astrum.repository.p2.internal.metadata.P2Attributes;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;

@Facet.Exposed
public interface P2ContentFacet
    extends ContentFacet
{
  Optional<Content> get(String path);

  boolean delete(String path);

  Content putBinary(P2Attributes p2Attributes, Payload payload) throws IOException;

  Content putBundle(P2Attributes p2Attributes, Payload payload) throws IOException;

  Content putMetadata(String path, AssetKind assetKind, Payload payload, Map<String, Object> attributes);

  List<FluentAsset> compositeMetadata();

  void deleteAll();
}
