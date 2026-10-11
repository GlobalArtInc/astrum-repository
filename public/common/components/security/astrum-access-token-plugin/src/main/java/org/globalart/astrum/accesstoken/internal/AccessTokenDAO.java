package org.globalart.astrum.accesstoken.internal;

import java.time.OffsetDateTime;
import java.util.Collection;
import java.util.Optional;

import org.globalart.astrum.datastore.api.DataAccess;

import org.apache.ibatis.annotations.Param;

/**
 * {@link AccessTokenData} access.
 */
public interface AccessTokenDAO
    extends DataAccess
{
  Collection<AccessTokenData> browse();

  Collection<AccessTokenData> browseByUser(@Param("username") String username);

  Optional<AccessTokenData> read(@Param("id") String id);

  Optional<AccessTokenData> findByHash(@Param("tokenHash") String tokenHash);

  int countByUserAndName(@Param("username") String username, @Param("name") String name);

  void create(AccessTokenData token);

  boolean delete(@Param("id") String id);

  int deleteByUser(@Param("username") String username);

  void updateLastUsed(@Param("id") String id, @Param("lastUsed") OffsetDateTime lastUsed);
}
