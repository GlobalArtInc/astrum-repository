
package org.globalart.astrum.datastore.api;

import java.util.Optional;

import org.globalart.astrum.common.entity.HasName;

/**
 * Generic CRUD {@link DataAccess} that accesses zero to many uniquely named entities of the same type.
 *
 * @since 3.19
 */
public interface NamedDataAccess<E extends HasName>
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
   * Retrieve the entity with the given name.
   */
  Optional<E> read(String name);

  /**
   * Update an existing entity.
   */
  boolean update(E entity);

  /**
   * Delete the entity with the given name.
   */
  boolean delete(String name);
}
