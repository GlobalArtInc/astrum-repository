
package org.globalart.astrum.repository.apt.datastore;

import java.io.IOException;
import java.io.InputStream;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.apt.internal.debian.PackageInfo;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

/**
 * Apt content facet
 *
 * @since 3.31
 */
@Facet.Exposed
public interface AptContentFacet
    extends ContentFacet
{
  String getDistribution();

  boolean isFlat();

  boolean isEnforceDistribution();

  Optional<FluentAsset> getAsset(String path);

  Optional<Content> get(String assetPath);

  FluentAsset put(String path, Payload content) throws IOException;

  FluentAsset put(String path, Payload payload, @Nullable PackageInfo packageInfo) throws IOException;

  FluentAsset findOrCreateMetadataAsset(TempBlob tempBlob, String path);

  TempBlob getTempBlob(Payload payload);

  TempBlob getTempBlob(InputStream in, @Nullable String contentType);

  void deleteAssetsByPrefix(String pathPrefix);

  Iterable<FluentAsset> getAptPackageAssets();
}
