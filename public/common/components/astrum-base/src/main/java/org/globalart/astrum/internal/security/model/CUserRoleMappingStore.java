
package org.globalart.astrum.internal.security.model;

import java.util.Optional;

import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
class CUserRoleMappingStore
    extends ConfigStoreSupport<CUserRoleMappingDAO>
{
  @Autowired
  protected CUserRoleMappingStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Transactional
  Iterable<CUserRoleMappingData> browse() {
    return dao().browse();
  }

  @Transactional
  void create(final CUserRoleMappingData mapping) {
    dao().create(mapping);
  }

  @Transactional
  Optional<CUserRoleMappingData> read(final String userId, final String source) {
    return dao().read(userId, source);
  }

  @Transactional
  boolean update(final CUserRoleMappingData mapping) {
    return dao().update(mapping);
  }

  @Transactional
  boolean delete(final String userId, final String source) {
    return dao().delete(userId, source);
  }
}
