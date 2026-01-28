
package org.globalart.astrum.blobstore.restore.datastore;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.content.store.AssetBlobStore;
import org.globalart.astrum.repository.content.store.FormatStoreManager;

import com.google.common.collect.ImmutableMap;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.MockedStatic;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.anyList;
import static org.mockito.Mockito.mockStatic;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.config.ConfigurationConstants.DATA_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AssetBlobRefFormatCheckTest

{
  public static final String MAVEN_2 = "maven2";

  public static final String DATASTORE_NAME = "astrum";

  @Mock
  private Format format;

  @Mock
  private Repository repository;

  @Mock
  private FormatStoreManager formatStoreManager;

  @Mock
  private AssetBlobStore<?> assetBlobStore;

  private AssetBlobRefFormatCheck underTest;

  private MockedStatic<QualifierUtil> mockedStatic;

  @Before
  public void setup() {
    mockedStatic = mockStatic(QualifierUtil.class);
    when(QualifierUtil.buildQualifierBeanMap(anyList())).thenReturn(Map.of(MAVEN_2, formatStoreManager));
    underTest = new AssetBlobRefFormatCheck(List.of(formatStoreManager));
  }

  @After
  public void tearDown() {
    mockedStatic.close();
  }

  @Test
  public void shouldBeTrueWhenAssetBlobRefHasNodeId() {
    mockRepository();
    mockAssetBlobStore(true);

    assertThat(underTest.isAssetBlobRefNotMigrated(repository), is(true));
  }

  @Test
  public void shouldBeFalseWhenAssetBlobRefDoesNotHaveNodeId() {
    mockRepository();
    mockAssetBlobStore(false);

    assertThat(underTest.isAssetBlobRefNotMigrated(repository), is(false));
  }

  private void mockRepository() {
    when(repository.getFormat()).thenReturn(format);
    when(format.getValue()).thenReturn(MAVEN_2);

    Configuration configuration = new ConfigurationData();
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    attributes.put(STORAGE, ImmutableMap.of(DATA_STORE_NAME, DATASTORE_NAME));
    configuration.setAttributes(attributes);
    when(repository.getConfiguration()).thenReturn(configuration);
  }

  private void mockAssetBlobStore(final boolean notMigrated) {
    when(formatStoreManager.assetBlobStore(DATASTORE_NAME)).thenReturn(assetBlobStore);
    when(assetBlobStore.notMigratedAssetBlobRefsExists()).thenReturn(notMigrated);
  }
}
