
package org.globalart.astrum.repository.content.internal;

import java.sql.Connection;

public interface DatabaseIntegrityChecker
{
  void checkAndRepair(Connection connection) throws Exception;
}
