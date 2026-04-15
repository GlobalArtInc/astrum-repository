package org.globalart.astrum.repository.golang.internal.content;

import java.util.Optional;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.golang.AssetKind;
import org.globalart.astrum.repository.golang.internal.metadata.GolangAttributes;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

@Facet.Exposed
public interface GoContentFacet
    extends ContentFacet
{
  Optional<FluentAsset> getAsset(String path);

  Optional<Content> getVersions(String escapedModule);

  Optional<Content> getInfo(GolangAttributes golangAttributes);

  FluentAsset upload(GolangAttributes golangAttributes, Payload payload);

  FluentAsset upload(GolangAttributes golangAttributes, TempBlob zipBlob);

  TempBlob ingest(Payload payload);

  FluentAsset saveComponentAndAsset(String path, Payload payload, AssetKind assetKind, GolangAttributes attributes);

  FluentAsset saveAsset(String path, Payload payload, AssetKind assetKind);
}
