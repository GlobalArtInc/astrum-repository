
package org.globalart.astrum.repository.security;

import java.util.Map;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.search.AssetSearchResult;
import org.globalart.astrum.repository.search.ComponentSearchResult;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.selector.ConstantVariableResolver;
import org.globalart.astrum.selector.PropertiesResolver;
import org.globalart.astrum.selector.VariableSource;
import org.globalart.astrum.selector.VariableSourceBuilder;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Adapts different contexts to variable resolvers
 *
 * @since 3.1
 */
public abstract class VariableResolverAdapterSupport
    implements VariableResolverAdapter
{
  protected static final String NAME = "name";

  protected static final String PATH = "path";

  protected static final String FORMAT = "format";

  @Override
  public VariableSource fromRequest(final Request request, final Repository repository) {
    VariableSourceBuilder builder = new VariableSourceBuilder();
    String path = request.getPath();
    builder.addResolver(new ConstantVariableResolver(path, PATH));
    builder.addResolver(new ConstantVariableResolver(repository.getFormat().getValue(), FORMAT));
    addFromRequest(builder, request);

    return builder.build();
  }

  protected abstract void addFromRequest(VariableSourceBuilder builder, Request request);

  @Override
  public VariableSource fromCoordinates(final String format, final String path, final Map<String, String> coordinates) {
    VariableSourceBuilder builder = new VariableSourceBuilder();
    builder.addResolver(new ConstantVariableResolver(checkNotNull(path), PATH));
    builder.addResolver(new ConstantVariableResolver(checkNotNull(format), FORMAT));

    addCoordinates(builder, coordinates);
    return builder.build();
  }

  @Override
  public VariableSource fromSourceLookup(final String format, final Map<String, Object> asset) {
    VariableSourceBuilder builder = new VariableSourceBuilder();
    builder.addResolver(
        new ConstantVariableResolver(checkNotNull((String) asset.get(NAME)), PATH));
    builder.addResolver(
        new ConstantVariableResolver(format, FORMAT));
    addFromSourceLookup(builder, asset);

    return builder.build();
  }

  @Override
  public VariableSource fromSearchResult(final ComponentSearchResult component, final AssetSearchResult asset) {
    VariableSourceBuilder builder = new VariableSourceBuilder();
    builder.addResolver(new ConstantVariableResolver(asset.getPath(), PATH));
    builder.addResolver(new ConstantVariableResolver(component.getFormat(), FORMAT));

    addFromSearchResults(builder, component, asset);

    return builder.build();
  }

  protected abstract void addFromSourceLookup(
      VariableSourceBuilder builder,
      Map<String, Object> asset);

  protected abstract void addFromSearchResults(
      VariableSourceBuilder builder,
      ComponentSearchResult component,
      AssetSearchResult asset);

  protected void addCoordinates(final VariableSourceBuilder builder, final Map<String, String> coordinates) {
    builder.addResolver(new PropertiesResolver<>("coordinate", coordinates));
  }

  @Override
  public VariableSource fromPath(final String path, final String format) {
    VariableSourceBuilder builder = new VariableSourceBuilder();
    builder.addResolver(new ConstantVariableResolver(path, PATH));
    builder.addResolver(new ConstantVariableResolver(format, FORMAT));

    return builder.build();
  }
}
