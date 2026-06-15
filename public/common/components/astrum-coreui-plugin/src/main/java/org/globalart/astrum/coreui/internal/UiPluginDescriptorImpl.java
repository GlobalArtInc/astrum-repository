
package org.globalart.astrum.coreui.internal;

import org.globalart.astrum.rapture.UiPluginDescriptor;
import org.globalart.astrum.rapture.UiPluginDescriptorSupport;

import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

/**
 * Rapture {@link UiPluginDescriptor} for {@code astrum-coreui-plugin}.
 *
 * @since 3.0
 */
@Component
@Order(Ordered.HIGHEST_PRECEDENCE + 100) // after astrum-rapture
public class UiPluginDescriptorImpl
    extends UiPluginDescriptorSupport
{
  public UiPluginDescriptorImpl() {
    super("astrum-coreui-plugin");
    setNamespace("NX.coreui");
    setConfigClassName("NX.coreui.app.PluginConfig");
  }
}
