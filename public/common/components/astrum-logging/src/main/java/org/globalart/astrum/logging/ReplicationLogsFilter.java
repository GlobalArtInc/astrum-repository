
package org.globalart.astrum.logging;

import org.slf4j.MDC;

import static org.globalart.astrum.logging.task.ReplicationTaskLogger.REPLICATION_DISCRIMINATOR_ID;

public class ReplicationLogsFilter
    extends TaskLogsFilter
{
  @Override
  protected boolean isExecutingInTask() {
    // if null then not running a replication task
    return MDC.get(REPLICATION_DISCRIMINATOR_ID) != null;
  }
}
