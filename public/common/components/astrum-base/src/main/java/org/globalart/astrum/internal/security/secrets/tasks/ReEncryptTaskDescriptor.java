
package org.globalart.astrum.internal.security.secrets.tasks;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.crypto.secrets.SecretsService.SECRETS_MIGRATION_VERSION;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

@AvailabilityVersion(from = SECRETS_MIGRATION_VERSION)
@Component
@Qualifier(ReEncryptTaskDescriptor.TYPE_ID)
public class ReEncryptTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "security.secrets.re-encrypt";

  public static final String EXPOSED_FLAG_VALUE = "${astrum.secrets.re-encrypt.task.expose:false}";

  public static final String VISIBLE_FLAG_VALUE = "${astrum.secrets.re-encrypt.task.visible:false}";

  @Autowired
  public ReEncryptTaskDescriptor(
      @Value(EXPOSED_FLAG_VALUE) final boolean exposed,
      @Value(VISIBLE_FLAG_VALUE) final boolean visible)
  {
    super(TYPE_ID,
        ReEncryptTask.class,
        "Admin - Re-encrypt secrets with the specified key",
        visible,
        exposed,
        true /* request recovery */);
  }
}
