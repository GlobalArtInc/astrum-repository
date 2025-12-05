
package org.globalart.astrum.internal.app;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.capability.CapabilitySupport;
import org.globalart.astrum.common.app.BaseUrlManager;

import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Base-URL capability.
 *
 * @since 3.0
 */
@Component(BaseUrlCapabilityDescriptor.TYPE_ID)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class BaseUrlCapability
    extends CapabilitySupport<BaseUrlCapabilityConfiguration>
{
  private final BaseUrlManager baseUrlManager;

  @Autowired
  public BaseUrlCapability(final BaseUrlManager baseUrlManager) {
    this.baseUrlManager = checkNotNull(baseUrlManager);
  }

  @Override
  protected BaseUrlCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new BaseUrlCapabilityConfiguration(properties);
  }

  @Override
  protected void onActivate(final BaseUrlCapabilityConfiguration config) throws Exception {
    baseUrlManager.setUrl(config.getUrl());
  }

  @Override
  protected void onPassivate(final BaseUrlCapabilityConfiguration config) throws Exception {
    baseUrlManager.setUrl(null);
  }

  @Override
  protected void onUpdate(BaseUrlCapabilityConfiguration config) throws Exception {
    super.onUpdate(config);
    baseUrlManager.setUrl(config.getUrl());
  }
}
