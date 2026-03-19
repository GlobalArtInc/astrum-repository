
package org.globalart.astrum.content.maven.internal;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.security.AssetVariableResolverSupport;
import org.globalart.astrum.repository.maven.MavenPath.Coordinates;
import org.globalart.astrum.repository.maven.MavenPathParser;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.maven.internal.utils.MavenVariableResolverAdapterUtil;
import org.globalart.astrum.repository.search.AssetSearchResult;
import org.globalart.astrum.repository.search.ComponentSearchResult;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.selector.VariableSourceBuilder;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Maven2 implementation will expose the groupId/artifactId/version/extension/classifier attributes when available.
 *
 * @since 3.25
 */
@Component
@Qualifier(Maven2Format.NAME)
public class MavenVariableResolverAdapter
    extends AssetVariableResolverSupport
{
  private final MavenPathParser mavenPathParser;

  @Autowired
  public MavenVariableResolverAdapter(@Qualifier(Maven2Format.NAME) final MavenPathParser mavenPathParser) {
    this.mavenPathParser = checkNotNull(mavenPathParser);
  }

  @Override
  protected void addFromRequest(final VariableSourceBuilder builder, final Request request) {
    addMavenCoordinates(builder, request.getPath());
  }

  @Override
  protected void addFromSourceLookup(
      final VariableSourceBuilder builder,
      final Map<String, Object> asset)
  {
    addMavenCoordinates(builder, (String) asset.get(NAME));
  }

  @Override
  protected void addFromSearchResults(
      final VariableSourceBuilder builder,
      final ComponentSearchResult component,
      final AssetSearchResult asset)
  {
    addMavenCoordinates(builder, asset.getPath());
  }

  @Override
  protected void addFromAsset(final VariableSourceBuilder builder, final FluentAsset asset) {
    addMavenCoordinates(builder, asset.path());
  }

  /**
   * Adds the Maven coordinates extracted from the specified path, if available.
   */
  private void addMavenCoordinates(final VariableSourceBuilder builder, final String path) {
    checkNotNull(builder);
    checkNotNull(path);
    Coordinates coords = mavenPathParser.parsePath(path).getCoordinates();

    if (coords != null) {
      addCoordinates(builder, MavenVariableResolverAdapterUtil.createCoordinateMap(coords));
    }
  }
}
