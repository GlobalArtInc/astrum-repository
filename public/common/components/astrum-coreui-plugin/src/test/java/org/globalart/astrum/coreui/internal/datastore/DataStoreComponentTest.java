
package org.globalart.astrum.coreui.internal.datastore;

import java.util.List;

import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.datastore.api.DataStoreConfiguration;
import org.globalart.astrum.datastore.api.DataStoreManager;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.security.RepositoryPermissionChecker;

import com.google.common.collect.ImmutableMap;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static java.util.Arrays.asList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class DataStoreComponentTest
{
  @Mock
  private DataStoreManager dataStoreManager;

  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private RepositoryPermissionChecker repositoryPermissionChecker;

  @Mock
  private DataStore<?> h2DataStore;

  @Mock
  private DataStore<?> postgresqlDataStore;

  private DataStoreComponent underTest;

  @Before
  public void setup() {
    DataStoreConfiguration contentConfig = new DataStoreConfiguration();
    contentConfig.setName("content");
    contentConfig.setType("jdbc");
    contentConfig.setSource("local");
    contentConfig.setAttributes(ImmutableMap.of("jdbcUrl", "jdbc:h2:som/datastore/url"));

    DataStoreConfiguration configConfig = new DataStoreConfiguration();
    configConfig.setName("config");
    configConfig.setType("jdbc");
    configConfig.setSource("local");
    configConfig.setAttributes(ImmutableMap.of("jdbcUrl", "jdbc:postgresql:some/datastore/url"));

    when(h2DataStore.getConfiguration()).thenReturn(contentConfig);
    when(postgresqlDataStore.getConfiguration()).thenReturn(configConfig);

    when(dataStoreManager.browse()).thenReturn(asList(h2DataStore, postgresqlDataStore));

    underTest = new DataStoreComponent(dataStoreManager, repositoryManager, repositoryPermissionChecker);
  }

  @Test
  public void testReadingDatabase() {
    List<DataStoreXO> dataStores = underTest.read();
    assertThat(dataStores, hasSize(2));
    assertThat(dataStores.get(0).getName(), is("content"));
    assertThat(dataStores.get(1).getName(), is("config"));
  }

  @Test
  public void testReadingH2Database() {
    List<DataStoreXO> dataStores = underTest.readH2();
    assertThat(dataStores, hasSize(1));
    assertThat(dataStores.get(0).getName(), is("content"));
  }
}
