
package org.globalart.astrum.common.decorator;

/**
 * Interface for objects that will be leveraged as a decorator. Needed for {@link DecoratorUtils} for the use case of
 * being able to access a particular type within the chain of decorator objects.
 *
 * @see DecoratorUtils
 * @since 3.8
 */
public interface DecoratedObject<T>
{
  /**
   * Get the wrapped object that this instance is holding
   */
  T getWrappedObject();
}
