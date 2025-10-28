
package org.globalart.astrum.common.log;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Prefix for use when logging actions while in dry-run mode (i.e. log actions, but make no changes).
 *
 * @since 3.6
 */
@Component
public class DryRunPrefix
{
  private final String prefix;

  @Autowired
  public DryRunPrefix(
      @Value("${astrum.log.dryrun.prefix:::DRY RUN:: }") final String prefix)
  {
    this.prefix = checkNotNull(prefix);
  }

  public String get() {
    return prefix;
  }
}
