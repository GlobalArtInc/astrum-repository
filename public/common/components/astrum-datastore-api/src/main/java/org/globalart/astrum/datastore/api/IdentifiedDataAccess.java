
package org.globalart.astrum.datastore.api;

import java.util.Optional;

import org.globalart.astrum.common.entity.HasStringId;

/**
 * Generic CRUD {@link DataAccess} that accesses zero to many externally identified entities of the same type.
 *
 * @since 3.21
 */
public interface IdentifiedDataAccess<E extends HasStringId>
    extends DataAccess
{
  /**
   * Browse existing entities.
   */
  Iterable<E> browse();

  /**
   * Create a new entity.
   */
  void create(E entity);

  /**
   * Retrieve the entity with the given id.
   */
  Optional<E> read(String id);

  /**
   * Update an existing entity.
   */
  boolean update(E entity);

  /**
   * Delete the entity with the given id.
   */
  boolean delete(String id);
}
