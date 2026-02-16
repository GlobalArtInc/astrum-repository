
package org.globalart.astrum.repository.httpbridge.legacy;

import java.util.Map;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.capability.CapabilitySupport;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Legacy URL capability.
 *
 * @since 3.7
 */
@Component(LegacyUrlCapabilityDescriptor.TYPE_ID)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class LegacyUrlCapability
    extends CapabilitySupport<LegacyUrlCapabilityConfiguration>
{
  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("NXRM2 style urls enabled")
    String description();
  }

  private static final Messages messages = I18N.create(Messages.class);

  @Override
  protected LegacyUrlCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new LegacyUrlCapabilityConfiguration();
  }

  @Override
  protected String renderDescription() throws Exception {
    if (context().isActive()) {
      return messages.description();
    }
    return null;
  }
}
