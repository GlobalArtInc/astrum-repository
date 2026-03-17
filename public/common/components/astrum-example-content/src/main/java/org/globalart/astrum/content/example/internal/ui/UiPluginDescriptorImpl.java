
package org.globalart.astrum.content.example.internal.ui;

import org.globalart.astrum.rapture.UiPluginDescriptorSupport;

import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

@Component
@Order(Ordered.HIGHEST_PRECEDENCE + 200)
public class UiPluginDescriptorImpl
    extends UiPluginDescriptorSupport
{
  public UiPluginDescriptorImpl() {
    super("astrum-example-content");
    setNamespace("NX.example");
    setConfigClassName("NX.example.app.PluginConfig");
  }
}
