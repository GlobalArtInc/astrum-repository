
package org.globalart.astrum.selector;

import java.util.Optional;
import java.util.Set;

/**
 * An interface for resolving variables to values.
 *
 * @since 3.0
 */
public interface VariableResolver
{
  /**
   * Resolve a given {@code variable} to its underlying value.
   *
   * @param variable to be resolved
   * @return resolved value
   */
  Optional<Object> resolve(String variable);

  /**
   * Get the set of variables that are resolvable by this resolver.
   *
   * @return the set of resolvable variables
   */
  Set<String> getVariableSet();
}
