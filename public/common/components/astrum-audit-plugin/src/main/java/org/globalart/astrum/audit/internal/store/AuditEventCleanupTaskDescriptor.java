
package org.globalart.astrum.audit.internal.store;

import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.stereotype.Component;

/**
 * Task descriptor for the audit event retention cleanup task.
 * <p>
 * Not exposed and not visible in the UI — the schedule is managed by
 * {@link AuditEventCleanupTaskManager}, which auto-creates it on startup with a daily
 * cron.
 */
@Component
@AvailabilityVersion(from = "1.0")
public class AuditEventCleanupTaskDescriptor
    extends TaskDescriptorSupport
{
  public static final String TYPE_ID = "audit.events.cleanup";

  public static final String TASK_NAME = "Admin - Cleanup old audit events";

  public AuditEventCleanupTaskDescriptor() {
    super(TYPE_ID,
        AuditEventCleanupTask.class,
        TASK_NAME,
        NOT_VISIBLE,
        NOT_EXPOSED);
  }
}
