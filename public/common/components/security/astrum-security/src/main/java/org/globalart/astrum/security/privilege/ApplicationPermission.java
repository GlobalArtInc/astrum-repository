
package org.globalart.astrum.security.privilege;

import java.util.Arrays;
import java.util.List;

import org.globalart.astrum.security.authz.WildcardPermission2;

import com.google.common.base.Splitter;
import com.google.common.collect.ImmutableList;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Application permission, using {@code astrum:} namespace.
 *
 * @since 3.0
 */
public class ApplicationPermission
    extends WildcardPermission2
{
  public static final String SYSTEM = "astrum";

  private static final char PART_SEPARATOR = ':';

  private static final Splitter PART_SPLITTER = Splitter.on(PART_SEPARATOR);

  private final String domain;

  private final List<String> actions;

  public ApplicationPermission(final String domain, final List<String> actions) {
    this.domain = checkNotNull(domain);
    this.actions = checkNotNull(actions);

    if (domain.indexOf(PART_SEPARATOR) < 0) {
      setParts(ImmutableList.of(SYSTEM, domain), actions);
    }
    else {
      // complex domain we need to split into parts
      setParts(ImmutableList.<String>builder()
          .add(SYSTEM)
          .addAll(PART_SPLITTER.split(domain))
          .build(), actions);
    }
  }

  public ApplicationPermission(final String domain, final String... actions) {
    this(domain, Arrays.asList(actions));
  }

  public String getDomain() {
    return domain;
  }

  public List<String> getActions() {
    return actions;
  }
}
