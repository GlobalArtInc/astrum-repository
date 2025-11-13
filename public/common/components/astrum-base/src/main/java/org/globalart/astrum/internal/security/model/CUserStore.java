
package org.globalart.astrum.internal.security.model;

import java.util.Optional;

import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.security.config.CUser;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
class CUserStore
    extends ConfigStoreSupport<CUserDAO>
{
  @Autowired
  protected CUserStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Transactional
  public Iterable<CUserData> browse() {
    return dao().browse();
  }

  @Transactional
  public Optional<CUser> read(final String id) {
    return dao().read(id)
        .map(CUser.class::cast);
  }

  @Transactional
  public void create(final CUserData user) {
    dao().create(user);
  }

  @Transactional
  public boolean update(final CUserData user) {
    return dao().update(user);
  }

  @Transactional
  public boolean delete(final String id) {
    return dao().delete(id);
  }
}
