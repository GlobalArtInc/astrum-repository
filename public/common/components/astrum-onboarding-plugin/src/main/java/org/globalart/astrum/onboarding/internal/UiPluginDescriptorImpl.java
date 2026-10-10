
package org.globalart.astrum.onboarding.internal;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.rapture.UiPluginDescriptor;
import org.globalart.astrum.rapture.UiPluginDescriptorSupport;

import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

/**
 * Rapture {@link UiPluginDescriptor} for {@code astrum-onboarding-plugin}.
 *
 * @since 3.17
 */
@Component
@Order(Ordered.HIGHEST_PRECEDENCE + 300) // after astrum-proui-plugin
public class UiPluginDescriptorImpl
    extends UiPluginDescriptorSupport
{
  @Autowired
  public UiPluginDescriptorImpl() {
    super("astrum-onboarding-plugin");
    setNamespace("AS.onboarding");
    setConfigClassName("AS.onboarding.app.PluginConfig");
  }
}
