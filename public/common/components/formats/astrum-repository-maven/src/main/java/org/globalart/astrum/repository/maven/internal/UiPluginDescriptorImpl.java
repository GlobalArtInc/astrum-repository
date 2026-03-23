
package org.globalart.astrum.repository.maven.internal;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.rapture.UiPluginDescriptorSupport;

import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

/**
 * Rapture {@link org.globalart.astrum.rapture.UiPluginDescriptor} for {@code astrum-repository-maven}.
 *
 * @since 3.15
 */
@Component
@Order(Ordered.HIGHEST_PRECEDENCE + 300) // after proui
public class UiPluginDescriptorImpl
    extends UiPluginDescriptorSupport
{
  @Autowired
  public UiPluginDescriptorImpl() {
    super("astrum-repository-maven");
    setHasStyle(false);
    setNamespace("NX.maven");
    setConfigClassName("NX.maven.app.PluginConfig");
  }
}
