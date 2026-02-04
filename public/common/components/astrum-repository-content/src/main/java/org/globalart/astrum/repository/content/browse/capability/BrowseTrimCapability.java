
package org.globalart.astrum.repository.content.browse.capability;

import java.util.Map;

import org.globalart.astrum.capability.CapabilitySupport;
import org.globalart.astrum.capability.Condition;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component(BrowseTrimCapabilityDescriptor.TYPE_ID)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class BrowseTrimCapability
    extends CapabilitySupport<BrowseTrimCapabilityConfiguration>
{
  private final BrowseTrimService browseTrimService;

  @Autowired
  public BrowseTrimCapability(final BrowseTrimService browseTrimService) {
    this.browseTrimService = checkNotNull(browseTrimService);
  }

  @Override
  protected BrowseTrimCapabilityConfiguration createConfig(final Map<String, String> properties) {
    return new BrowseTrimCapabilityConfiguration(properties);
  }

  @Override
  protected void onActivate(final BrowseTrimCapabilityConfiguration config) throws Exception {
    browseTrimService.setPostgresqlTrimEnabled(config.isPostgresqlTrimEnabled());
    browseTrimService.setBatchTrimEnabled(config.isBatchTrimEnabled());
  }

  @Override
  protected void onPassivate(final BrowseTrimCapabilityConfiguration config) throws Exception {
    browseTrimService.setPostgresqlTrimEnabled(false);
    browseTrimService.setBatchTrimEnabled(false);
  }

  @Override
  protected void onRemove(final BrowseTrimCapabilityConfiguration config) throws Exception {
    browseTrimService.setPostgresqlTrimEnabled(false);
    browseTrimService.setBatchTrimEnabled(false);
  }

  @Override
  public Condition activationCondition() {
    return conditions().capabilities().passivateCapabilityDuringUpdate();
  }
}
