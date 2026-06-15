
package org.globalart.astrum.coreui.internal.datastore;

import java.util.List;
import java.util.Map;
import java.util.stream.StreamSupport;
import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.datastore.api.DataStoreManager;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.rapture.StateContributor;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.security.RepositoryPermissionChecker;
import org.globalart.astrum.security.privilege.ApplicationPermission;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.google.common.collect.ImmutableMap;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;

import static java.util.Collections.singletonList;
import static java.util.stream.Collectors.toList;
import static org.globalart.astrum.security.BreadActions.READ;
import org.springframework.stereotype.Component;

/**
 * DataStore {@link org.globalart.astrum.extdirect.DirectComponent}.
 */
@Component
@DirectAction(action = "coreui_Datastore")
public class DataStoreComponent
    extends DirectComponentSupport
    implements StateContributor
{
  private static final String DATASTORES_FIELD = "datastores";

  private static final String JDBCURL_FIELD = "jdbcUrl";

  private static final String DATASTORES_PERMISSION = "datastores";

  private final DataStoreManager dataStoreManager;

  private final RepositoryManager repositoryManager;

  private final RepositoryPermissionChecker repositoryPermissionChecker;

  private final boolean enabled;

  @Autowired
  public DataStoreComponent(
      final DataStoreManager dataStoreManager,
      final RepositoryManager repositoryManager,
      final RepositoryPermissionChecker repositoryPermissionChecker)
  {
    this.dataStoreManager = dataStoreManager;
    this.repositoryManager = repositoryManager;
    this.repositoryPermissionChecker = repositoryPermissionChecker;
    this.enabled = true;
  }

  @Override
  @Nullable
  public Map<String, Object> getState() {
    return ImmutableMap.of(DATASTORES_FIELD, enabled);
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<DataStoreXO> read() {
    repositoryPermissionChecker.ensureUserHasAnyPermissionOrAdminAccess(
        singletonList(new ApplicationPermission(DATASTORES_PERMISSION, READ)),
        READ,
        repositoryManager.browse());
    return StreamSupport.stream(dataStoreManager.browse().spliterator(), false)
        .map(this::asDataStoreXO)
        .collect(toList());
  }

  @DirectMethod
  @Timed
  @ExceptionMetered
  public List<DataStoreXO> readH2() {
    repositoryPermissionChecker.ensureUserHasAnyPermissionOrAdminAccess(
        singletonList(new ApplicationPermission(DATASTORES_PERMISSION, READ)),
        READ,
        repositoryManager.browse());
    return StreamSupport.stream(dataStoreManager.browse().spliterator(), false)
        .filter(
            dataStore -> dataStore.getConfiguration()
                .getAttributes()
                .getOrDefault(JDBCURL_FIELD, "")
                .startsWith("jdbc:h2:"))
        .map(this::asDataStoreXO)
        .collect(toList());
  }

  private DataStoreXO asDataStoreXO(final DataStore<?> dataStore) {
    DataStoreXO dataStoreXO = new DataStoreXO();
    dataStoreXO.setName(dataStore.getConfiguration().getName());
    return dataStoreXO;
  }
}
