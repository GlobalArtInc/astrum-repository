
package org.globalart.astrum.rapture.internal;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.rapture.UiPluginDescriptor;
import org.globalart.astrum.rapture.UiPluginDescriptorSupport;

import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

/**
 * Rapture {@link UiPluginDescriptor} for {@code astrum-rapture}.
 *
 * @since 3.0
 */
@Component
@Order(Ordered.HIGHEST_PRECEDENCE) // always load first
public class UiPluginDescriptorImpl
    extends UiPluginDescriptorSupport
{
  @Autowired
  public UiPluginDescriptorImpl() {
    super("astrum-rapture");
    setConfigClassName("NX.app.PluginConfig");
  }
}
