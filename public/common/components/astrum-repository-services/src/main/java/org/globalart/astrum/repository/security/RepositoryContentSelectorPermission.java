
package org.globalart.astrum.repository.security;

import java.util.List;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.security.authz.WildcardPermission2;
import org.globalart.astrum.selector.SelectorConfiguration;

import com.google.common.collect.ImmutableList;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Repository content selector permission.
 *
 * @since 3.1
 */
public class RepositoryContentSelectorPermission
    extends WildcardPermission2
{
  public static final String SYSTEM = "astrum";

  public static final String DOMAIN = "repository-content-selector";

  private final String selector;

  private final String format;

  private final String name;

  private final List<String> actions;

  public RepositoryContentSelectorPermission(
      final String selector,
      final String format,
      final String name,
      final List<String> actions)
  {
    this.selector = checkNotNull(selector);
    this.format = checkNotNull(format);
    this.name = checkNotNull(name);
    this.actions = checkNotNull(actions);

    setParts(ImmutableList.of(SYSTEM, DOMAIN, selector, format, name), actions);
  }

  public RepositoryContentSelectorPermission(
      final SelectorConfiguration selector,
      final Repository repository,
      final List<String> actions)
  {
    this(selector.getName(), repository.getFormat().getValue(), repository.getName(), actions);
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
