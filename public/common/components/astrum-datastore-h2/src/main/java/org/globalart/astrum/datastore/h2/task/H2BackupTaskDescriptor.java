
package org.globalart.astrum.datastore.h2.task;

import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.common.upgrade.AvailabilityVersion;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.scheduling.TaskDescriptor;
import org.globalart.astrum.scheduling.TaskDescriptorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.formfields.FormField.MANDATORY;

/**
 * A {@link TaskDescriptor} for backing up an embedded H2 datastore.
 *
 * The task descriptor is only visible/exposed when running against H2. On
 * PostgreSQL-backed installations the H2 backup task has no meaningful
 * implementation (the underlying MyBatis data store rejects {@code backup()}
 * with {@code UnsupportedOperationException}), so hiding the descriptor keeps
 * it out of the scheduler UI and REST API.
 */
@AvailabilityVersion(from = "1.0")
@Component
public class H2BackupTaskDescriptor
    extends TaskDescriptorSupport
{
  static final String TYPE_ID = "h2.backup.task";

  static final String LOCATION = "location";

  private final DatabaseCheck databaseCheck;

  @Autowired
  public H2BackupTaskDescriptor(final DatabaseCheck databaseCheck) {
    super(TYPE_ID, H2BackupTask.class, "Admin - Backup H2 Database", VISIBLE, EXPOSED,
        new StringTextFormField(LOCATION, "Location",
            "Specify a directory for the database backup",
            MANDATORY));
    this.databaseCheck = checkNotNull(databaseCheck);
  }

  @Override
  public boolean isVisible() {
    return !databaseCheck.isPostgresql();
  }

  @Override
  public boolean isExposed() {
    return !databaseCheck.isPostgresql();
  }
}
