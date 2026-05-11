
package org.globalart.astrum.rapture;

import java.util.Collections;
import java.util.List;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Support for {@link UiPluginDescriptor} implementations.
 *
 * @since 3.0
 * @deprecated as of 3.20, replaced by {@link org.globalart.astrum.ui.UiPluginDescriptorSupport}
 *
 *             Note that this class should only be implemented if your plugin is including UI content using ExtJs. If
 *             you are
 *             using react (as the UI is moving towards) use {@link org.globalart.astrum.ui.UiPluginDescriptor}
 */
@Deprecated
public abstract class UiPluginDescriptorSupport
    implements UiPluginDescriptor
{
  private boolean hasStyle = true;

  private boolean hasScript = true;

  private String namespace;

  private String configClassName;

  /**
   * Artifact ID of UI-contributing plugin. Used to resolve the location of its contributed web-resources.
   */
  private final String pluginId;

  public UiPluginDescriptorSupport(final String artifactId) {
    this.pluginId = checkNotNull(artifactId);
  }

  @Override
  public String getPluginId() {
    return pluginId;
  }

  @Override
  public boolean hasStyle() {
    return hasStyle;
  }

  public void setHasStyle(final boolean hasStyle) {
    this.hasStyle = hasStyle;
  }

  @Override
  public boolean hasScript() {
    return hasScript;
  }

  public void setHasScript(final boolean hasScript) {
    this.hasScript = hasScript;
  }

  @Override
  @Nullable
  public String getNamespace() {
    return namespace;
  }

  public void setNamespace(@Nullable final String namespace) {
    this.namespace = namespace;
  }

  @Override
  @Nullable
  public String getConfigClassName() {
    return configClassName;
  }

  public void setConfigClassName(final String configClassName) {
    this.configClassName = configClassName;
  }

  @Override
  public List<String> getScripts(final boolean isDebug) {
    return Collections.emptyList();
  }
}
