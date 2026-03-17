
package org.globalart.astrum.content.example.internal.recipe;

import java.io.IOException;
import java.util.Optional;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.content.example.ExampleContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.collect.ImmutableList;

import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Provides persistent content for an 'example' format.
 *
 * @since 3.24
 */
@Component
@Qualifier(ExampleFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ExampleContentFacetImpl
    extends ContentFacetSupport
    implements ExampleContentFacet
{
  private static final Iterable<HashAlgorithm> HASHING = ImmutableList.of(SHA256);

  @Autowired
  public ExampleContentFacetImpl(
      @Qualifier(ExampleFormat.NAME) final FormatStoreManager formatStoreManager)
  {
    super(formatStoreManager);
  }

  @Override
  public Optional<Content> get(final String path) {
    return assets().path(path).find().map(FluentAsset::download);
  }

  @Override
  public Content put(final String path, final Payload content) throws IOException {
    try (TempBlob blob = blobs().ingest(content, HASHING)) {
      return assets().path(path).blob(blob).save().markAsCached(content).download();
    }
  }

  @Override
  public boolean delete(final String path) throws IOException {
    return assets().path(path).find().map(FluentAsset::delete).orElse(false);
  }
}
