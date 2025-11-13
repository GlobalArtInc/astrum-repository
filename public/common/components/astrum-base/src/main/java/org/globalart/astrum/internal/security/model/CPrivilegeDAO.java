
package org.globalart.astrum.internal.security.model;

import java.util.List;
import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.datastore.api.IdentifiedDataAccess;
import org.globalart.astrum.security.config.CPrivilege;

import org.apache.ibatis.annotations.Param;

/**
 * {@link CPrivilegeData} access.
 *
 * @since 3.21
 */
public interface CPrivilegeDAO
    extends IdentifiedDataAccess<CPrivilegeData>
{

  /**
   * Retrieve the entity with the given name.
   */
  Optional<CPrivilege> readByName(String name);

  /**
   * Update an entity by its name
   */
  boolean updateByName(CPrivilegeData entity);

  /**
   * Delete an entity with the given name.
   */
  boolean deleteByName(String name);

  /**
   * Find privileges by their ids.
   *
   * @param ids a set of privilege ids.
   * @return a list of {@link CPrivilege}
   */
  List<CPrivilege> findByIds(@Param("ids") Set<String> ids);
}
