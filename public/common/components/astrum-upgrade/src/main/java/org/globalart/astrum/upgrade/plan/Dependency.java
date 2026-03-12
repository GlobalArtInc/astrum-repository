
package org.globalart.astrum.upgrade.plan;

/**
 * Dependency.
 *
 * @since 3.1
 */
public interface Dependency<T>
{
  /**
   * Returns {@code true} if given satisfies this dependency.
   */
  boolean satisfiedBy(T other);
}
