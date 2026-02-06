
package org.globalart.astrum.repository.content.security;

import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.selector.VariableSource;

/**
 * Generate a variable source from a persisted asset, to be used for content selector evaluation
 *
 * @since 3.29
 */
public interface AssetVariableResolver
    extends VariableResolverAdapter
{
  VariableSource fromAsset(FluentAsset asset);
}
