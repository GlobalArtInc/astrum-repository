
package org.globalart.astrum.security.config;

import static com.google.common.base.Preconditions.checkState;

/**
 * Helper to build a {@link CRole} instance.
 *
 * @since 3.0
 */
public class CRoleBuilder
{
  private final CRole model;

  CRoleBuilder(final CRole model) {
    this.model = model;
  }

  public CRoleBuilder id(final String id) {
    model.setId(id);
    return this;
  }

  public CRoleBuilder name(final String name) {
    model.setName(name);
    return this;
  }

  public CRoleBuilder description(final String description) {
    model.setDescription(description);
    return this;
  }

  public CRoleBuilder privilege(final String privilege) {
    model.addPrivilege(privilege);
    return this;
  }

  public CRoleBuilder role(final String role) {
    model.addRole(role);
    return this;
  }

  public CRoleBuilder readOnly(final boolean readOnly) {
    model.setReadOnly(readOnly);
    return this;
  }

  public CRole create() {
    checkState(model.getId() != null, "Missing: id");
    if (model.getName() == null) {
      model.setName(model.getId());
    }
    if (model.getDescription() == null) {
      model.setDescription(model.getId());
    }
    return model;
  }
}
