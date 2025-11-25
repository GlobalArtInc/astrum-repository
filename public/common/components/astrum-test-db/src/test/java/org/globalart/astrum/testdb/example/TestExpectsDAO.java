
package org.globalart.astrum.testdb.example;

import org.globalart.astrum.datastore.api.DataAccess;
import org.globalart.astrum.datastore.api.Expects;

@Expects(TestItemDAO.class)
public interface TestExpectsDAO
    extends DataAccess
{
  // no additional behaviour
}
