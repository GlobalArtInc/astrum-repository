
package org.globalart.astrum.internal.security.secrets.task;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * This task should only be run on systems where {@link SecretsService.SECRETS_MIGRATION_VERSION} is valid, for
 * as it's intended to be invoked in the context of an upgrade step in the same context as the database upgrade we use
 * 1.0 for the availability version.
 */
@AvailabilityVersion(from = "1.0")
@Component
public class SecretsMigrationTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "secrets.migration";

  private static final String EXPOSED_FLAG_VALUE = "${astrum.secrets.migration.exposed:false}";

  private static final String VISIBLE_FLAG_VALUE = "${astrum.secrets.migration.visible:false}";

  @Autowired
  public SecretsMigrationTaskDescriptor(
      @Value(EXPOSED_FLAG_VALUE) final boolean exposed,
      @Value(VISIBLE_FLAG_VALUE) final boolean visible)
  {
    super(TYPE_ID,
        SecretsMigrationTask.class,
        "Secrets - Migrate secrets",
        visible,
        exposed);
  }
}
