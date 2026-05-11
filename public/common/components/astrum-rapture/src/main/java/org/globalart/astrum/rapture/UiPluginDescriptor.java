
package org.globalart.astrum.rapture;

import java.util.List;

import javax.annotation.Nullable;

/**
 * Rapture ExtJS UI plugin descriptor.
 *
 * @since 3.0
 * @deprecated as of 3.20, replaced by {@link org.globalart.astrum.ui.UiPluginDescriptor}
 *
 *             Note that this class should only be implemented if your plugin is including UI content using ExtJs. If
 *             you are
 *             using react (as the UI is moving towards) use {@link org.globalart.astrum.ui.UiPluginDescriptor}
 */
@Deprecated
public interface UiPluginDescriptor
{
  /**
   * The plugin identifier. This is normally the POM artifactId.
   * This is used to generate references to Javascript and CSS sources.
   */
  String getPluginId();

  boolean hasStyle();

  boolean hasScript();

  /**
   * Extjs application plugin namespace.
   */
  @Nullable
  String getNamespace();

  /**
   * The Extjs class name of the {@code PluginConfig} for the plugin.
   */
  @Nullable
  String getConfigClassName();

  /**
   * @since 3.20
   * @return a list of script files that should be included on the page
   */
  List<String> getScripts(final boolean isDebug);
}
