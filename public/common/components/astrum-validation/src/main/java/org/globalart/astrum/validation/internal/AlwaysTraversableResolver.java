
package org.globalart.astrum.validation.internal;

import java.lang.annotation.ElementType;

import jakarta.validation.Path;
import jakarta.validation.Path.Node;
import jakarta.validation.TraversableResolver;

/**
 * Always {@link TraversableResolver} to disable JPA reachability.
 *
 * @since 3.0
 */
public class AlwaysTraversableResolver
    implements TraversableResolver
{
  /**
   * return {@code true}
   */
  public boolean isCascadable(
      final Object traversableObject,
      final Node traversableProperty,
      final Class<?> rootBeanType,
      final Path pathToTraversableObject,
      final ElementType elementType)
  {
    return true;
  }

  /**
   * return {@code true}
   */
  public boolean isReachable(
      final Object traversableObject,
      final Node traversableProperty,
      final Class<?> rootBeanType,
      final Path pathToTraversableObject,
      final ElementType elementType)
  {
    return true;
  }
}
