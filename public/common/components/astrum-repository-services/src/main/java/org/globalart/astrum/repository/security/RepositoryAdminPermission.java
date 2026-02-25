
package org.globalart.astrum.repository.security;

import java.util.Arrays;
import java.util.List;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.security.authz.WildcardPermission2;

import com.google.common.collect.ImmutableList;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Repository administration permission.
 *
 * @since 3.0
 */
public class RepositoryAdminPermission
    extends WildcardPermission2
{
  public static final String SYSTEM = "astrum";

  public static final String DOMAIN = "repository-admin";

  private final String format;

  private final String name;

  private final List<String> actions;

  public RepositoryAdminPermission(final String format, final String name, final List<String> actions) {
    this.format = checkNotNull(format);
    this.name = checkNotNull(name);
    this.actions = checkNotNull(actions);

    setParts(ImmutableList.of(SYSTEM, DOMAIN, format, name), actions);
  }

  public RepositoryAdminPermission(final String format, final String name, final String... actions) {
    this(format, name, Arrays.asList(actions));
  }

  /**
   * @since 3.17
   */
  public RepositoryAdminPermission(final Repository repository, final String... actions) {
    this(repository.getFormat().getValue(), repository.getName(), Arrays.asList(actions));
  }

  public String getFormat() {
    return format;
  }

  public String getName() {
    return name;
  }

  public List<String> getActions() {
    return actions;
  }
}
