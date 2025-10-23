
package org.globalart.astrum.common.app;

import javax.annotation.Nullable;
import jakarta.inject.Named;

/**
 * Helper to lookup components in global context.
 * In a few places, components need to be looked up by class-name and need to use the uber class-loader to resolve
 * classes.
 * This helper contains this logic in one place for re-use.
 *
 * @since 3.0
 */
public interface GlobalComponentLookupHelper
{
  /**
   * Lookup a component by class-name.
   *
   * @return Component reference, or {@code null} if the component was not found.
   */
  @Nullable
  Object lookup(String className);

  /**
   * Lookup a component by {@link Class}.
   *
   * @return Component reference, or {@code null} if the component was not found.
   * @since 3.6.1
   */
  <T> T lookup(Class<T> clazz);

  /**
   * Lookup a component by {@link Class} and @{@link Named} name.
   *
   * @return Component reference, or {@code null} if the component was not found.
   * @since 3.6.1
   */
  <T> T lookup(Class<T> clazz, String name);

  /**
   * Lookup a type by class-name.
   *
   * @return Type reference, or {@code null} if the type was not found.
   */
  @Nullable
  Class<?> type(String className);
}
