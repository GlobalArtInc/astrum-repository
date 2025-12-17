
package org.globalart.astrum.security.config;

import java.util.Arrays;

import org.globalart.astrum.security.config.memory.MemoryCPrivilege;

import com.google.common.base.Joiner;

import static com.google.common.base.Preconditions.checkState;

/**
 * Helper to build a {@link CPrivilege} instance.
 *
 * @since 3.0
 */
public class CPrivilegeBuilder
{
  private final CPrivilege model;

  public CPrivilegeBuilder() {
    this.model = new MemoryCPrivilege();
  }

  public CPrivilegeBuilder type(final String type) {
    model.setType(type);
    return this;
  }

  public CPrivilegeBuilder id(final String id) {
    model.setId(id);
    return this;
  }

  public CPrivilegeBuilder name(final String name) {
    model.setName(name);
    return this;
  }

  public CPrivilegeBuilder description(final String description) {
    model.setDescription(description);
    return this;
  }

  public CPrivilegeBuilder readOnly(final boolean readOnly) {
    model.setReadOnly(readOnly);
    return this;
  }

  public CPrivilegeBuilder property(final String name, final String value) {
    model.setProperty(name, value);
    return this;
  }

  public CPrivilegeBuilder property(final String name, final Iterable<String> values) {
    return property(name, Joiner.on(',').join(values));
  }

  public CPrivilegeBuilder property(final String name, final String... values) {
    return property(name, Arrays.asList(values));
  }

  public CPrivilege create() {
    checkState(model.getType() != null, "Missing: type");
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
