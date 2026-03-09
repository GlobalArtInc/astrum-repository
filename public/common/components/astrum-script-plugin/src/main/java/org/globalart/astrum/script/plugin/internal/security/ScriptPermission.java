
package org.globalart.astrum.script.plugin.internal.security;

import java.util.List;
import java.util.Objects;

import org.globalart.astrum.security.authz.WildcardPermission2;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Script permission.
 * Allows for fine-grained permissions on Scripts based on their name.
 *
 * @since 3.0
 */
public class ScriptPermission
    extends WildcardPermission2
{
  public static final String SYSTEM = "astrum";

  public static final String DOMAIN = "script";

  private final String name;

  private final List<String> actions;

  public ScriptPermission(String name, List<String> actions) {
    this.name = checkNotNull(name);
    this.actions = checkNotNull(actions);

    setParts(List.of(SYSTEM, DOMAIN, name), actions);
  }

  public String getName() {
    return name;
  }

  public List<String> getActions() {
    return actions;
  }

  @Override
  public boolean equals(final Object o) {
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    if (!super.equals(o)) {
      return false;
    }
    ScriptPermission that = (ScriptPermission) o;
    return Objects.equals(name, that.name) && Objects.equals(actions, that.actions);
  }

  @Override
  public int hashCode() {
    return Objects.hash(super.hashCode(), name, actions);
  }
}
