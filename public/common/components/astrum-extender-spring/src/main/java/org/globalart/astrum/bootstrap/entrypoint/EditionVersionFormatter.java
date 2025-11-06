
package org.globalart.astrum.bootstrap.entrypoint;

import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEditionSelector;
import org.globalart.astrum.common.app.ApplicationVersion;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Utility class for formatting edition and version information.
 */
public final class EditionVersionFormatter
{
  private static final Logger log = LoggerFactory.getLogger(EditionVersionFormatter.class);

  private EditionVersionFormatter() {
    // utility class
  }

  /**
   * Formats the edition ID and version for display.
   *
   * @param astrumEditionSelector the edition selector
   * @param applicationVersion the application version (may be null)
   * @return formatted string in the form "editionId" or "editionId/version"
   */
  public static String formatEditionAndVersion(
      final AstrumEditionSelector astrumEditionSelector,
      @Nullable final ApplicationVersion applicationVersion)
  {
    String editionId = Optional.ofNullable(astrumEditionSelector.getCurrent())
        .map(edition -> edition.getId())
        .orElseGet(() -> {
          log.warn("Unable to determine edition ID, using 'unknown' as fallback");
          return "unknown";
        });
    return applicationVersion != null
        ? editionId + "/" + applicationVersion.getVersion()
        : editionId;
  }
}
