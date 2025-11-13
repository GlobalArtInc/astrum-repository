
package org.globalart.astrum.internal.security.model;

import java.util.Optional;

import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
class CRoleStore
    extends ConfigStoreSupport<CRoleDAO>
{
  @Autowired
  protected CRoleStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Transactional
  Iterable<CRoleData> browse() {
    return dao().browse();
  }

  @Transactional
  void create(final CRoleData role) {
    dao().create(role);
  }

  @Transactional
  boolean delete(final String id) {
    return dao().delete(id);
  }

  @Transactional
  Optional<CRoleData> read(final String id) {
    return dao().read(id);
  }

  @Transactional
  Iterable<CRoleData> readByIds(final java.util.Collection<String> ids) {
    if (ids == null || ids.isEmpty()) {
      return java.util.Collections.emptyList();
    }
    return dao().readByIds(ids);
  }

  @Transactional
  boolean update(final CRoleData roleData) {
    return dao().update(roleData);
  }
}
