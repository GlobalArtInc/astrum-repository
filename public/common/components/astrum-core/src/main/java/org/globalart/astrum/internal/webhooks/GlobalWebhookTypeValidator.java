
package org.globalart.astrum.internal.webhooks;

import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.validation.ConstraintValidatorSupport;
import org.globalart.astrum.webhooks.GlobalWebhook;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.springframework.beans.factory.config.ConfigurableBeanFactory.SCOPE_PROTOTYPE;

/**
 * Validates that every entry in the configured list of Global Webhook event names corresponds to a
 * discovered {@link GlobalWebhook} bean.
 * <p>
 * Historically this validator also gated the {@code firewall_quarantine} event on the presence of
 * an enabled {@code clm} capability and an enabled {@code firewall.audit} capability with
 * {@code quarantine=true}. Those capability-based prerequisites were removed as part of ASTRUM-53667:
 * firewall configuration is now stored on each repository (see
 * {@code FirewallCapabilityToRepositoryConfigMigrationStep_2_143}) and the {@code firewall.audit}
 * capability rows no longer exist post-migration, so that check always failed. The event pipeline
 * (see {@code FirewallContributedHandler}) already gates on
 * {@code FirewallConfigurationHelper.isFirewallEnabled(repository)} and
 * {@code clmConnector.isEnabled()} at fire time, so a webhook subscribed to a
 * {@code firewall_quarantine} event that no repository will ever raise is harmless.
 */
@Component
@Scope(SCOPE_PROTOTYPE)
public class GlobalWebhookTypeValidator
    extends ConstraintValidatorSupport<GlobalWebhookType, List<String>>
{
  private final List<GlobalWebhook> globalWebhooks;

  @Autowired
  public GlobalWebhookTypeValidator(final List<GlobalWebhook> globalWebhooks) {
    this.globalWebhooks = checkNotNull(globalWebhooks);
  }

  @Override
  public boolean isValid(final List<String> names, final ConstraintValidatorContext constraintValidatorContext) {
    if (names == null || names.isEmpty()) {
      return true; // empty list is valid
    }

    Set<String> webhookNames =
        globalWebhooks.stream().map(GlobalWebhook::getName).collect(Collectors.toUnmodifiableSet());

    return webhookNames.containsAll(names);
  }
}
