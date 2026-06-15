
package org.globalart.astrum.coreui.internal;

import java.util.List;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.ui.UiPluginDescriptor;
import org.globalart.astrum.ui.UiUtil;

import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

/**
 * {@link UiPluginDescriptor} for {@code astrum-coreui-plugin} react code.
 *
 * @since 3.22
 */
@Component
@Order(Ordered.HIGHEST_PRECEDENCE + 100) // after astrum-rapture
public class CoreUiReactPluginDescriptorImpl
    implements UiPluginDescriptor
{
  private final List<String> scripts;

  private final List<String> debugScripts;

  private final List<String> styles;

  @Autowired
  public CoreUiReactPluginDescriptorImpl(final UiUtil uiUtil) {
    scripts = List.of(uiUtil.getPathForFile("astrum-coreui-bundle.js"));
    debugScripts = List.of(uiUtil.getPathForFile("astrum-coreui-bundle.debug.js"));
    styles = List.of(uiUtil.getPathForFile("astrum-coreui-bundle.css"));
  }

  @Override
  public String getName() {
    return "astrum-coreui-plugin";
  }

  @Nullable
  @Override
  public List<String> getScripts(final boolean isDebug) {
    return isDebug ? debugScripts : scripts;
  }

  @Nullable
  @Override
  public List<String> getStyles() {
    return styles;
  }
}
