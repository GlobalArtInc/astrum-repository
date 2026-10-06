
package org.globalart.astrum.self.hosted.internal.capability.proxy;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.capability.CapabilitySupport;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.capability.CapabilityType.capabilityType;

/**
 * Capability to control forwarded header processing at runtime.
 *
 * <p>
 * This capability allows enabling/disabling the processing of forwarded headers
 * (X-Forwarded-For, X-Forwarded-Proto, etc.) without restarting the server.
 * </p>
 */
@Component(ForwardedRequestCapability.TYPE_ID)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ForwardedRequestCapability
    extends CapabilitySupport<ForwardedRequestCapabilityConfiguration>
{
  public static final String TYPE_ID = "http.forwarded";

  public static final CapabilityType TYPE = capabilityType(TYPE_ID);

  interface Messages
      extends MessageBundle
  {
    @DefaultMessage("HTTP Forwarded Headers")
    String name();

    @DefaultMessage("HTTP")
    String category();

    @DefaultMessage("Disabled")
    String disabled();

    @DefaultMessage("Enabled")
    String enabled();
  }

  static final Messages messages = I18N.create(Messages.class);

  private final ForwardedRequestCustomizerManager manager;

  @Autowired
  public ForwardedRequestCapability(final ForwardedRequestCustomizerManager manager) {
    this.manager = checkNotNull(manager);
  }

  @Override
  protected ForwardedRequestCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new ForwardedRequestCapabilityConfiguration(properties);
  }

  @Override
  @Nullable
  protected String renderDescription() {
    if (context().isActive()) {
      return messages.enabled();
    }
    return messages.disabled();
  }

  @Override
  public Condition activationCondition() {
    return conditions().capabilities().passivateCapabilityDuringUpdate();
  }

  @Override
  protected void onActivate(final ForwardedRequestCapabilityConfiguration config) {
    manager.setEnabled(config.isEnabled());
  }

  @Override
  protected void onPassivate(final ForwardedRequestCapabilityConfiguration config) {
    manager.setEnabled(false);
  }

  @Override
  protected void onUpdate(final ForwardedRequestCapabilityConfiguration config) {
    manager.setEnabled(config.isEnabled());
  }
}
