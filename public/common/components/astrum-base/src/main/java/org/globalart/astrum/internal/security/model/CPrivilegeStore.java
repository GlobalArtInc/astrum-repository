
package org.globalart.astrum.internal.security.model;

import java.util.List;
import java.util.Optional;
import java.util.Set;

import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
class CPrivilegeStore
    extends ConfigStoreSupport<CPrivilegeDAO>
{
  @Autowired
  protected CPrivilegeStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Transactional
  Iterable<CPrivilegeData> browse() {
    return dao().browse();
  }

  @Transactional
  void create(final CPrivilegeData privilege) {
    dao().create(privilege);
  }

  @Transactional
  boolean delete(final String id) {
    return dao().delete(id);
  }

  @Transactional
  boolean deleteByName(final String name) {
    return dao().deleteByName(name);
  }

  @Transactional
  List<CPrivilege> findByIds(final Set<String> ids) {
    return dao().findByIds(ids);
  }

  @Transactional
  Optional<CPrivilege> read(final String id) {
    return dao().read(id)
        .map(CPrivilege.class::cast);
  }

  @Transactional
  Optional<CPrivilege> readByName(final String name) {
    return dao().readByName(name);
  }

  @Transactional
  boolean update(final CPrivilegeData privilege) {
    return dao().update(privilege);
  }

  @Transactional
  boolean updateByName(final CPrivilegeData privilege) {
    return dao().updateByName(privilege);
  }
}
