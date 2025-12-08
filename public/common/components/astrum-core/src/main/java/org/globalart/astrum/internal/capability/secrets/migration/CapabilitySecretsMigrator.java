
package org.globalart.astrum.internal.capability.secrets.migration;

import java.util.List;
import java.util.function.Predicate;
import java.util.stream.Collectors;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.formfields.Encrypted;
import org.globalart.astrum.scheduling.CancelableHelper;
import org.globalart.astrum.security.secrets.SecretsMigrator;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Migrates existing records from the legacy secret format to the new secret format.
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CapabilitySecretsMigrator
    implements SecretsMigrator
{
  private final CapabilityRegistry capabilityRegistry;

  @Autowired
  public CapabilitySecretsMigrator(final CapabilityRegistry capabilityRegistry) {
    this.capabilityRegistry = checkNotNull(capabilityRegistry);
  }

  @Override
  public void migrate() {
    List<CapabilityReference> maybeEncrypted = capabilityRegistry.getAll()
        .stream()
        .filter(containsEncryptedField())
        .collect(Collectors.toList());

    for (CapabilityReference ref : maybeEncrypted) {
      CancelableHelper.checkCancellation();
      capabilityRegistry.migrateSecrets(ref, this::isLegacyEncryptedString);
    }
  }

  private Predicate<CapabilityReference> containsEncryptedField() {
    return reference -> reference.context()
        .descriptor()
        .formFields()
        .stream()
        .anyMatch(f -> f instanceof Encrypted);
  }
}
