
package org.globalart.astrum.content.maven.upgrade;

import java.util.Optional;

import org.springframework.stereotype.Component;

/**
 * Re-index for update to search normalization format.
 *
 * @since 3.37
 */
@Component
public class MavenUpgrade_1_7 // NOSONAR
    extends MavenIndexUpgrade
{
  @Override
  public Optional<String> version() {
    return Optional.of("1.7");
  }
}
