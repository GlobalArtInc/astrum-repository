
package org.globalart.astrum.repository.content.security.internal;

import java.util.Map;

import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.security.AssetVariableResolverSupport;
import org.globalart.astrum.repository.search.AssetSearchResult;
import org.globalart.astrum.repository.search.ComponentSearchResult;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.selector.VariableSourceBuilder;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Provides only 'path' and 'format' variables for content selector evaluation.
 * This is the correct approach for all repository formats.
 *
 * <p>
 * <b>Important:</b> Do NOT create format-specific VariableResolverAdapters that parse coordinates.
 * CSEL only supports path and format variables. Coordinate-based selectors were removed in ASTRUM-27129.
 *
 * <p>
 * <b>Usage:</b> New formats should inject this using {@code @Qualifier("simple")}.
 *
 * <p>
 * <b>Maven/NuGet Exception:</b> MavenVariableResolverAdapter and NugetVariableResolverAdapter exist
 * only for backward compatibility with pre-2021 JEXL selectors. Do not copy this pattern.
 */
@Component
@Qualifier("simple")
public class SimpleVariableResolverAdapter
    extends AssetVariableResolverSupport
{
  @Override
  protected void addFromRequest(final VariableSourceBuilder builder, final Request request) {
    // no-op the simple impl just allows for the path/format variable resolvers in the support class
  }

  @Override
  protected void addFromSourceLookup(final VariableSourceBuilder builder, final Map<String, Object> asset) {
    // no-op the simple impl just allows for the path/format variable resolvers in the support class
  }

  @Override
  protected void addFromSearchResults(
      final VariableSourceBuilder builder,
      final ComponentSearchResult component,
      final AssetSearchResult asset)
  {
    // no-op the simple impl just allows for the path/format variable resolvers in the support class
  }

  @Override
  protected void addFromAsset(final VariableSourceBuilder builder, final FluentAsset asset) {
    // no-op the simple impl just allows for the path/format variable resolvers in the support class
  }
}
