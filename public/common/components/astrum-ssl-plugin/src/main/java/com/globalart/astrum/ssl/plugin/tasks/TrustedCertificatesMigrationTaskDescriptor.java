
package com.globalart.astrum.ssl.plugin.tasks;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;
import org.springframework.stereotype.Component;

@AvailabilityVersion(from = "1.1")
@Component
public class TrustedCertificatesMigrationTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "trusted.certificates.migration";

  @Autowired
  public TrustedCertificatesMigrationTaskDescriptor() {
    super(TYPE_ID, TrustedCertificatesMigrationTask.class,
        "Migrate trusted certificates Task",
        TaskDescriptorSupport.NOT_VISIBLE,
        TaskDescriptorSupport.NOT_EXPOSED);
  }
}
