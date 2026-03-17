
package org.globalart.astrum.content.raw.internal.recipe;

import java.io.IOException;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.content.raw.RawContentFacet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.maintenance.ContentMaintenanceFacet;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.raw.RawCoordinatesHelper;
import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

import org.springframework.beans.factory.annotation.Qualifier;

/**
 * A {@link RawContentFacet} that persists to a {@link ContentFacet}.
 *
 * @since 3.24
 */
@Component
@Qualifier(RawFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RawContentFacetImpl
    extends ContentFacetSupport
    implements RawContentFacet
{
  @Autowired
  public RawContentFacetImpl(
      @Qualifier(RawFormat.NAME) final FormatStoreManager formatStoreManager)
  {
    super(formatStoreManager);
  }

  @Override
  public Optional<Content> get(final String path) throws IOException {
    return assets().path(path).find().map(FluentAsset::download);
  }

  @Override
  public FluentAsset getOrCreateAsset(
      final Repository repository,
      final String componentName,
      final String componentGroup,
      final String assetName)
  {
    return assets().path(componentName)
        .component(components()
            .name(componentName)
            .namespace(RawCoordinatesHelper.getGroup(componentName))
            .getOrCreate())
        .save();
  }

  @Override
  public Content put(final String path, final Payload content) throws IOException {
    try (TempBlob blob = blobs().ingest(content, HASHING)) {
      return assets()
          .path(path)
          .component(components()
              .name(path)
              .namespace(RawCoordinatesHelper.getGroup(path))
              .getOrCreate())
          .blob(blob)
          .save()
          .markAsCached(content)
          .download();
    }
  }

  @Override
  public boolean delete(final String path) throws IOException {
    return assets().path(path)
        .find()
        .map(asset -> repository().facet(ContentMaintenanceFacet.class).deleteAsset(asset).contains(path))
        .orElse(false);
  }
}
