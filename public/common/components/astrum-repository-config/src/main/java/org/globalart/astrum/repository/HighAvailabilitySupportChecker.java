
package org.globalart.astrum.repository;

import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

import org.globalart.astrum.common.node.NodeAccess;

import static java.lang.String.format;
import static org.globalart.astrum.common.property.SystemPropertiesHelper.getBoolean;

/**
 * Verifies availability of the repository dependent on clustering mode of Astrum and the clustering configuration
 * property of the repository.
 *
 * @since 3.17
 */
public abstract class HighAvailabilitySupportChecker
{
  private final boolean isAstrumClustered;

  private final Map<String, Boolean> formatHAStates = new ConcurrentHashMap<>();

  public HighAvailabilitySupportChecker(final NodeAccess nodeAccess) {
    isAstrumClustered = nodeAccess.isClustered();

    // Formats enabled by default
    getEnabledFormats().forEach(format -> formatHAStates.put(format, queryFormat(format, true)));
  }

  /**
   * Get formats enabled by default.
   *
   * @return a list of formats which are enabled by default in the cluster mode.
   */
  protected abstract List<String> getEnabledFormats();

  public boolean isSupported(final String formatName) {
    boolean formatEnabled = formatHAStates.computeIfAbsent(formatName, f -> queryFormat(formatName, false));
    return !isAstrumClustered || formatEnabled;
  }

  private boolean queryFormat(final String formatName, final boolean defaultValue) {
    return getBoolean(format("astrum.%s.ha.supported", formatName), defaultValue);
  }
}
