
package org.globalart.astrum.datastore.api;

/**
 * Each {@link DataAccess} mapping consists of a schema and associated operations.
 *
 * @since 3.19
 */
public interface DataAccess
{
  /**
   * Attempts to create the underlying schema mapping in the owning data store.
   */
  void createSchema();

  /**
   * Optional binding point to allow format-specific DAOs to extend their schema.
   */
  default void extendSchema() {
    // no changes to base schema...
  }
}
