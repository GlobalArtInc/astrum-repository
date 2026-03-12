
package org.globalart.astrum.upgrade.datastore.internal;

import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.upgrade.UpgradeService;
import org.globalart.astrum.upgrade.datastore.DeploymentValidator;
import org.globalart.astrum.upgrade.datastore.UpgradeManager;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.CLUSTERED_ZERO_DOWNTIME_ENABLED;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.UPGRADE;

/**
 * Default datastore {@link UpgradeService}.
 *
 * @since 3.29
 */
@Component
@ConditionalOnProperty(name = CLUSTERED_ZERO_DOWNTIME_ENABLED, havingValue = "false", matchIfMissing = true)
@Order(Ordered.HIGHEST_PRECEDENCE + 1)
@ManagedLifecycle(phase = UPGRADE)
public class UpgradeServiceImpl
    extends StateGuardLifecycleSupport
    implements UpgradeService
{
  private final Optional<DeploymentValidator> deploymentValidator;

  private final UpgradeManager upgradeManager;

  @Autowired
  public UpgradeServiceImpl(
      @Nullable final DeploymentValidator deploymentValidator,
      final UpgradeManager upgradeManager)
  {
    this.upgradeManager = checkNotNull(upgradeManager);
    this.deploymentValidator = Optional.ofNullable(deploymentValidator);
  }

  @Override
  protected void doStart() throws Exception {
    deploymentValidator.ifPresent(DeploymentValidator::validate);

    upgradeManager.migrate();
  }
}
