
package org.globalart.astrum.internal.security.apikey.task;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.1")
@Component
public class ReEncryptPrincipalsTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "reencrypt.principals";

  private static final String EXPOSED_FLAG_VALUE = "${astrum.reencrypt.principals.exposed:false}";

  private static final String VISIBLE_FLAG_VALUE = "${astrum.reencrypt.principals.visible:false}";

  @Autowired
  public ReEncryptPrincipalsTaskDescriptor(
      @Value(EXPOSED_FLAG_VALUE) final boolean exposed,
      @Value(VISIBLE_FLAG_VALUE) final boolean visible)
  {
    super(TYPE_ID,
        ReEncryptPrincipalsTask.class,
        "Re-encryption Principals",
        exposed,
        visible);
  }
}
