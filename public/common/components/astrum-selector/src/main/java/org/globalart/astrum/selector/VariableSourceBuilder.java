
package org.globalart.astrum.selector;

import java.util.ArrayList;
import java.util.List;

/**
 * A builder for building a {@link VariableSource} by adding any number
 * of {@link VariableResolver}'s.
 *
 * @since 3.0
 */
public class VariableSourceBuilder
{
  private List<VariableResolver> resolvers = new ArrayList<>();

  public VariableSourceBuilder addResolver(final VariableResolver resolver) {
    resolvers.add(resolver);
    return this;
  }

  /**
   * Build the {@link VariableSource}.
   *
   * @return immutable variable source
   */
  public VariableSource build() {
    return new VariableSource(resolvers);
  }
}
