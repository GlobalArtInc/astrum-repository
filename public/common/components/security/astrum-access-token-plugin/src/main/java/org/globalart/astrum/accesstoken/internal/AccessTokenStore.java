package org.globalart.astrum.accesstoken.internal;

import java.time.OffsetDateTime;
import java.util.Collection;
import java.util.Optional;

import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * Persists {@link AccessTokenData}.
 */
@Component
public class AccessTokenStore
    extends ConfigStoreSupport<AccessTokenDAO>
{
  @Autowired
  public AccessTokenStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Transactional
  public Collection<AccessTokenData> browse() {
    return dao().browse();
  }

  @Transactional
  public Collection<AccessTokenData> browseByUser(final String username) {
    return dao().browseByUser(username);
  }

  @Transactional
  public Optional<AccessTokenData> read(final String id) {
    return dao().read(id);
  }

  @Transactional
  public Optional<AccessTokenData> findByHash(final String tokenHash) {
    return dao().findByHash(tokenHash);
  }

  @Transactional
  public boolean exists(final String username, final String name) {
    return dao().countByUserAndName(username, name) > 0;
  }

  @Transactional
  public void create(final AccessTokenData token) {
    dao().create(token);
  }

  @Transactional
  public boolean delete(final String id) {
    return dao().delete(id);
  }

  @Transactional
  public int deleteByUser(final String username) {
    return dao().deleteByUser(username);
  }

  @Transactional
  public void updateLastUsed(final String id, final OffsetDateTime lastUsed) {
    dao().updateLastUsed(id, lastUsed);
  }
}
