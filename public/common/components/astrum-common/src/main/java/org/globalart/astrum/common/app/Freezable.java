
package org.globalart.astrum.common.app;

import static com.google.common.base.Preconditions.checkState;

/**
 * Some kind of writable component that can be temporarily frozen.
 *
 * @since 3.21
 */
public interface Freezable
{
  /**
   * Freezes the component, disallowing writes.
   */
  void freeze();

  /**
   * Unfreezes the component, allowing writes.
   */
  void unfreeze();

  /**
   * Is this component currently frozen?
   */
  boolean isFrozen();

  /**
   * Ensures that this component is currently readable.
   */
  default void checkReadable() {
    // assume readable by default
  }

  /**
   * Ensures that this component is currently writable.
   */
  default void checkWritable() {
    checkState(!isFrozen());
  }
}
