
package org.globalart.astrum.datastore.api;

import java.util.Optional;

/**
 * Generic {@link DataAccess} that only accesses one single entity.
 *
 * @since 3.19
 */
public interface SingletonDataAccess<E>
    extends DataAccess
{
  /**
   * Get current state of the entity if set.
   */
  Optional<E> get();

  /**
   * Set a new state for the entity.
   */
  void set(E entity);

  /**
   * Clears any set state.
   */
  void clear();
}
