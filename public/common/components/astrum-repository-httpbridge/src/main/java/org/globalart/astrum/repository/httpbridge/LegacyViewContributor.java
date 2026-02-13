
package org.globalart.astrum.repository.httpbridge;

import org.globalart.astrum.repository.httpbridge.internal.LegacyViewServlet;

/**
 * Legacy view contributor to {@link LegacyViewServlet}.
 *
 * @since 3.7
 */
public interface LegacyViewContributor
{
  LegacyViewConfiguration contribute();
}
