
package org.globalart.astrum.ui;

import java.util.List;

/**
 * Rapture UI plugin descriptor.
 *
 * Using component priority to determine inclusion order. Ordering is important to properly load the UI.
 *
 * @since 3.20
 */
public interface UiPluginDescriptor
{
  String getName();

  /**
   * @return a list of script files that should be included on the page (this is used for non-extjs plugins)
   */
  List<String> getScripts(final boolean isDebug);

  /**
   * @return a list of stylesheets that should be included on the page (this is used for non-extjs plugins)
   */
  List<String> getStyles();
}
