
package org.globalart.astrum.coreui.internal;

import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.rapture.StateContributor;

import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.DATASTORE_IS_POSTGRESQL;

@Component
public class DatastoreStateContributor
    implements StateContributor
{
  private final DatabaseCheck dbCheck;

  private Map<String, Object> state;

  @Autowired
  public DatastoreStateContributor(final DatabaseCheck dbCheck) {
    this.dbCheck = checkNotNull(dbCheck);
  }

  @Override
  public Map<String, Object> getState() {
    if (state == null) {
      state = Map.of(
          "astrum.datastore.enabled", true,
          "astrum.datastore.developer", false,
          DATASTORE_IS_POSTGRESQL, dbCheck.isPostgresql());
    }
    return state;
  }
}
