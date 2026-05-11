
package org.globalart.astrum.rapture.internal;

import java.util.Collections;
import java.util.List;

import javax.annotation.Nullable;

import org.globalart.astrum.ui.UiPluginDescriptor;
import org.globalart.astrum.ui.UiUtil;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

/**
 * {@link UiPluginDescriptor} for {@code astrum-rapture} react code.
 *
 * @since 3.25
 */
@Component
@Order(Ordered.HIGHEST_PRECEDENCE) // always load first
public class UiReactPluginDescriptorImpl
    implements UiPluginDescriptor
{
  private final List<String> scripts;

  private final List<String> debugScripts;

  @Autowired
  public UiReactPluginDescriptorImpl(final UiUtil uiUtil) {
    scripts = List.of(uiUtil.getPathForFile("astrum-rapture-bundle.js"));
    debugScripts = List.of(uiUtil.getPathForFile("astrum-rapture-bundle.debug.js"));
  }

  @Override
  public String getName() {
    return "astrum-rapture";
  }

  @Nullable
  @Override
  public List<String> getScripts(final boolean isDebug) {
    return isDebug ? debugScripts : scripts;
  }

  @Nullable
  @Override
  public List<String> getStyles() {
    return Collections.emptyList();
  }
}
