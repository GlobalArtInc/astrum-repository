
package org.globalart.astrum.rapture.internal.settings;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.capability.CapabilitySupport;
import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.rapture.UiSettingsManager;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Rapture Settings capability.
 *
 * @since 3.0
 */
@Component(SettingsCapabilityDescriptor.TYPE_ID)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class SettingsCapability
    extends CapabilitySupport<SettingsCapabilityConfiguration>
{

  private final UiSettingsManager rapture;

  private final SessionTimeoutManager sessionTimeoutManager;

  @Autowired
  public SettingsCapability(
      final UiSettingsManager rapture,
      final SessionTimeoutManager sessionTimeoutManager)
  {
    this.rapture = checkNotNull(rapture);
    this.sessionTimeoutManager = checkNotNull(sessionTimeoutManager);
  }

  @Override
  protected SettingsCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new SettingsCapabilityConfiguration(properties);
  }

  @Override
  protected void onActivate(final SettingsCapabilityConfiguration config) throws Exception {
    rapture.setSettings(config);
    sessionTimeoutManager.updateTimeout(config.getSessionTimeout());
  }

  @Override
  protected void onUpdate(final SettingsCapabilityConfiguration config) throws Exception {
    super.onUpdate(config);
    sessionTimeoutManager.updateTimeout(config.getSessionTimeout());
  }

  @Override
  protected void onPassivate(final SettingsCapabilityConfiguration config) throws Exception {
    rapture.resetSettings();
  }

  @Override
  protected void onRemove(final SettingsCapabilityConfiguration config) throws Exception {
    rapture.resetSettings();
  }

  @Override
  public Condition activationCondition() {
    return conditions().capabilities().passivateCapabilityDuringUpdate();
  }

}
