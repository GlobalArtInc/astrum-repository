
package org.globalart.astrum.repository.content.upgrades;

import java.util.Optional;

import org.globalart.astrum.repository.content.search.upgrade.SearchIndexUpgrade;
import org.springframework.stereotype.Component;

/**
 * Re-index search for all formats.
 */
@Component
public class SearchIndexUpgrade_1_25
    extends SearchIndexUpgrade
{
  @Override
  public Optional<String> version() {
    return Optional.of("1.25");
  }
}
