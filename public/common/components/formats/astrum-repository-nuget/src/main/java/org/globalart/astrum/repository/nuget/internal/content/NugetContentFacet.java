package org.globalart.astrum.repository.nuget.internal.content;

import java.io.IOException;
import java.util.List;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.nuget.internal.AssetKind;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetPackageMetadata;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

@Facet.Exposed
public interface NugetContentFacet
    extends ContentFacet
{
  Optional<Content> get(String path);

  Optional<FluentAsset> findAsset(String path);

  TempBlob ingest(Payload payload);

  Content putPackage(TempBlob blob, NugetPackageMetadata metadata, @Nullable Payload original);

  Content putPackage(String path, Payload payload) throws IOException;

  Content putMetadata(String path, Payload payload, AssetKind kind) throws IOException;

  List<FluentComponent> findComponents(String id);

  List<NugetPackageMetadata> findPackages(String id);

  Optional<FluentComponent> findComponent(String id, String version);

  List<NugetPackageMetadata> allPackages();

  boolean deletePackage(String id, String version);

  Optional<byte[]> nuspec(String id, String version) throws IOException;
}
