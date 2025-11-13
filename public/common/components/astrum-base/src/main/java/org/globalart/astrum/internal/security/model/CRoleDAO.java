
package org.globalart.astrum.internal.security.model;

import java.util.Collection;

import org.globalart.astrum.datastore.api.IdentifiedDataAccess;

import org.apache.ibatis.annotations.Param;

/**
 * {@link CRoleData} access.
 *
 * @since 3.21
 */
public interface CRoleDAO
    extends IdentifiedDataAccess<CRoleData>
{
  /**
   * Read multiple roles by their IDs in a single query.
   *
   * @param ids collection of role IDs to fetch
   * @return iterable of role data found (may be fewer than requested if some IDs don't exist)
   */
  Iterable<CRoleData> readByIds(@Param("ids") Collection<String> ids);
}
