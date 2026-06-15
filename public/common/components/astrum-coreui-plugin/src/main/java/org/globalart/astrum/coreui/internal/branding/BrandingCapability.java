
package org.globalart.astrum.coreui.internal.branding;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.capability.CapabilitySupport;
import org.globalart.astrum.capability.Condition;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Branding capability.
 *
 * @since 3.0
 */
@Component(BrandingCapabilityDescriptor.TYPE_ID)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class BrandingCapability
    extends CapabilitySupport<BrandingCapabilityConfiguration>
{

  private final Branding branding;

  @Autowired
  public BrandingCapability(final Branding branding) {
    this.branding = checkNotNull(branding);
  }

  @Override
  protected BrandingCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new BrandingCapabilityConfiguration(properties);
  }

  @Override
  protected void onActivate(final BrandingCapabilityConfiguration config) throws Exception {
    branding.set(config);
  }

  @Override
  protected void onPassivate(final BrandingCapabilityConfiguration config) throws Exception {
    branding.reset();
  }

  @Override
  protected void onRemove(final BrandingCapabilityConfiguration config) throws Exception {
    branding.reset();
  }

  @Override
  public Condition activationCondition() {
    return conditions().capabilities().passivateCapabilityDuringUpdate();
  }

}
