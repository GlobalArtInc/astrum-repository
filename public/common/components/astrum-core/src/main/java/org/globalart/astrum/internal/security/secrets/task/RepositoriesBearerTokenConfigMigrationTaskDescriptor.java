
package org.globalart.astrum.internal.security.secrets.task;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * Task descriptor for bearer token configuration migration task.
 */
@AvailabilityVersion(from = "1.0")
@Component
public class RepositoriesBearerTokenConfigMigrationTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "repository.bearer.token.config.migration";

  private static final String NAME = "Migrate bearer token repository configurations";

  public RepositoriesBearerTokenConfigMigrationTaskDescriptor(
      final @Value("${astrum.repository.bearer.migration:false}") boolean enabled)
  {
    super(TYPE_ID, RepositoriesBearerTokenConfigMigrationTask.class, NAME, enabled, enabled);
  }
}
