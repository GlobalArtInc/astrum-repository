
package org.globalart.astrum.internal.security.model;

import java.util.Optional;

import org.globalart.astrum.datastore.api.DataAccess;

import org.apache.ibatis.annotations.Param;

import static org.globalart.astrum.common.text.Strings2.lower;
import static org.globalart.astrum.security.config.SecuritySourceUtil.isCaseInsensitiveSource;

/**
 * {@link CUserRoleMappingData} access.
 *
 * @since 3.21
 */
public interface CUserRoleMappingDAO
    extends DataAccess
{
  Iterable<CUserRoleMappingData> browse();

  void create(CUserRoleMappingData mapping);

  Optional<CUserRoleMappingData> read(
      @Param("userId") String userId, // usual case-sensitive userId, non-null when doing usual search
      @Param("userLo") String userLo, // lowercase userId, non-null when doing case-insensitive search
      @Param("source") String source);

  default Optional<CUserRoleMappingData> read(String userId, String source) {
    return isCaseInsensitiveSource(source) ? read(null, lower(userId), source) : read(userId, null, source);
  }

  boolean update(CUserRoleMappingData mapping);

  boolean delete(
      @Param("userId") String userId, // usual case-sensitive userId, non-null when doing usual search
      @Param("userLo") String userLo, // lowercase userId, non-null when doing case-insensitive search
      @Param("source") String source);

  default boolean delete(String userId, String source) {
    return isCaseInsensitiveSource(source) ? delete(null, lower(userId), source) : delete(userId, null, source);
  }
}
