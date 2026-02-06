
package org.globalart.astrum.repository.content.security;

import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.security.VariableResolverAdapterSupport;
import org.globalart.astrum.selector.ConstantVariableResolver;
import org.globalart.astrum.selector.VariableSource;
import org.globalart.astrum.selector.VariableSourceBuilder;

/**
 * Adapts persisted assets to variable resolvers.
 *
 * @since 3.29
 */
public abstract class AssetVariableResolverSupport
    extends VariableResolverAdapterSupport
    implements AssetVariableResolver
{
  @Override
  public VariableSource fromAsset(final FluentAsset asset) {
    VariableSourceBuilder builder = new VariableSourceBuilder();
    builder.addResolver(new ConstantVariableResolver(asset.path(), PATH));
    builder.addResolver(new ConstantVariableResolver(asset.repository().getFormat().getValue(), FORMAT));
    addFromAsset(builder, asset);

    return builder.build();
  }

  @Override
  public VariableSource fromPath(final String path, final String format) {
    VariableSourceBuilder builder = new VariableSourceBuilder();
    builder.addResolver(new ConstantVariableResolver(path, PATH));
    builder.addResolver(new ConstantVariableResolver(format, FORMAT));

    return builder.build();
  }

  protected abstract void addFromAsset(VariableSourceBuilder builder, FluentAsset asset);
}
