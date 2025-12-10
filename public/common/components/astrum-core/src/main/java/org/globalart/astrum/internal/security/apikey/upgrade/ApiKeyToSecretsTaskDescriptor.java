
package org.globalart.astrum.internal.security.apikey.upgrade;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.internal.security.apikey.upgrade.ApiKeyToSecretsTask.MESSAGE;
import static org.globalart.astrum.internal.security.apikey.upgrade.ApiKeyToSecretsTask.TYPE_ID;
import org.springframework.stereotype.Component;

/**
 * This task should only be run on systems where {@link SecretsService.SECRETS_MIGRATION_VERSION} is valid, for
 * as its intended to be invoked in the context of an upgrade step in the same context as the database upgrade we use
 * 1.0 for the availability version.
 */
@AvailabilityVersion(from = "1.0")
@Component
public class ApiKeyToSecretsTaskDescriptor
    extends TaskDescriptorSupport
{
  @Autowired
  public ApiKeyToSecretsTaskDescriptor(
      @Value("${astrum.upgrade.apikey.secrets:false}") final boolean exposed)
  {
    super(TYPE_ID, ApiKeyToSecretsTask.class, MESSAGE, true, exposed);
  }
}
