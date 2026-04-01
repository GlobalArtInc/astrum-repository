
package org.globalart.astrum.repository.pypi.content;

import java.io.InputStream;
import java.util.List;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

@Facet.Exposed
public interface PypiContentFacet
    extends ContentFacet
{
  Optional<FluentAsset> getAsset(String path);

  boolean delete(String path);

  FluentAsset saveAsset(String packagePath, FluentComponent component, String assetKind, TempBlob tempBlob);

  FluentAsset saveAsset(String packagePath, String assetKind, TempBlob tempBlob);

  boolean isComponentExists(String name);

  List<FluentAsset> assetsByComponentName(String name);

  FluentComponent findOrCreateComponent(String name, String version, String normalizedName);

  TempBlob getTempBlob(Payload payload);

  TempBlob getTempBlob(InputStream content, @Nullable String contentType);
}
