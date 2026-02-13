
package org.globalart.astrum.repository.content.store.internal.migration;

import com.google.common.collect.ImmutableList;
import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import org.globalart.astrum.blobstore.api.BlobRef;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.entity.ContinuationAware;
import org.globalart.astrum.datastore.mybatis.ContinuationArrayList;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.store.AssetBlobData;
import org.globalart.astrum.repository.content.store.AssetBlobStore;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.scheduling.TaskConfiguration;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.MockedStatic;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyList;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.mockStatic;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.content.store.internal.AssetBlobCleanupTaskDescriptor.CONTENT_STORE_FIELD_ID;
import static org.globalart.astrum.repository.content.store.internal.AssetBlobCleanupTaskDescriptor.FORMAT_FIELD_ID;
import static org.globalart.astrum.repository.content.store.internal.AssetBlobCleanupTaskDescriptor.TYPE_ID;

/**
 * Tests for {@link AssetBlobRefMigrationTask}
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class AssetBlobRefMigrationTaskTest
{
  private static final int READ_ASSETS_BATCH_SIZE = 100;

  @Mock
  private FormatStoreManager formatStoreManager;

  @Mock
  private AssetBlobStore<?> assetBlobStore;

  private MockedStatic<QualifierUtil> mockedStatic;

  @Before
  public void setup() {
    mockedStatic = mockStatic(QualifierUtil.class);
    ContinuationArrayList<AssetBlobData> firstPage = new ContinuationArrayList<>();
    firstPage.add(newNotMigratedAssetBlobData());
    firstPage.add(newNotMigratedAssetBlobData());
    firstPage.add(newNotMigratedAssetBlobData());
    when(((ContinuationAware) firstPage.get(firstPage.size() - 1)).nextContinuationToken()).thenReturn("NEXT");

    ContinuationArrayList<AssetBlobData> lastPage = new ContinuationArrayList<>();
    lastPage.add(newNotMigratedAssetBlobData());
    lastPage.add(newNotMigratedAssetBlobData());
    when(lastPage.get(lastPage.size() - 1).nextContinuationToken()).thenReturn("EOL");

    ContinuationArrayList<AssetBlobData> emptyPage = new ContinuationArrayList<>();

    when(assetBlobStore.notMigratedAssetBlobRefsExists()).thenReturn(true);

    when(assetBlobStore.browseAssetsWithLegacyBlobRef(READ_ASSETS_BATCH_SIZE, null))
        .thenReturn((Continuation) firstPage);
    when(assetBlobStore.browseAssetsWithLegacyBlobRef(READ_ASSETS_BATCH_SIZE, "NEXT"))
        .thenReturn((Continuation) lastPage);
    when(assetBlobStore.browseAssetsWithLegacyBlobRef(READ_ASSETS_BATCH_SIZE, "EOL"))
        .thenReturn((Continuation) emptyPage);

    when(assetBlobStore.updateBlobRefs(any())).thenReturn(true);

    when(formatStoreManager.assetBlobStore("content")).thenReturn(assetBlobStore);
  }

  @After
  public void teardown() {
    mockedStatic.close();
  }

  @Test
  public void testUpdateDuplicatedBlobRef() {
    AssetBlobRefMigrationTask assetBlobRefMigrationTask =
        new AssetBlobRefMigrationTask(List.of(), 3);

    String uuid = UUID.randomUUID().toString();
    AssetBlobData blobData1 = new AssetBlobData();
    AssetBlobData blobData2 = new AssetBlobData();
    AssetBlobData blobData3 = new AssetBlobData();
    blobData1.setBlobRef(new BlobRef("default", uuid));
    blobData2.setBlobRef(new BlobRef("default", uuid));
    blobData3.setBlobRef(new BlobRef("default", uuid));

    Collection<AssetBlob> blobs = ImmutableList.of(blobData1, blobData2, blobData3);

    assetBlobRefMigrationTask.updateDuplicatedBlobRef(blobs);

    assertTrue(blobs.stream()
        .map(AssetBlob::blobRef)
        .map(BlobRef::getBlob)
        .noneMatch(blob -> blob.equals(uuid)));
  }

  @Test
  public void testAssetBlobRefMigration() throws Exception {
    when(QualifierUtil.buildQualifierBeanMap(anyList())).thenReturn(Map.of("raw", formatStoreManager));
    AssetBlobRefMigrationTask underTest = new AssetBlobRefMigrationTask(List.of(formatStoreManager),
        READ_ASSETS_BATCH_SIZE);

    TaskConfiguration taskConfiguration = new TaskConfiguration();
    taskConfiguration.setString(FORMAT_FIELD_ID, "raw");
    taskConfiguration.setString(CONTENT_STORE_FIELD_ID, "content");
    taskConfiguration.setId(UUID.randomUUID().toString());
    taskConfiguration.setTypeId(TYPE_ID);
    underTest.configure(taskConfiguration);

    underTest.execute();

    verify(assetBlobStore).browseAssetsWithLegacyBlobRef(READ_ASSETS_BATCH_SIZE, null);
    verify(assetBlobStore).browseAssetsWithLegacyBlobRef(READ_ASSETS_BATCH_SIZE, "NEXT");
    verify(assetBlobStore).browseAssetsWithLegacyBlobRef(READ_ASSETS_BATCH_SIZE, "EOL");

    verify(assetBlobStore, times(2)).updateBlobRefs(any());
  }

  @Test
  public void testMigrationCompletesSuccessfully() throws Exception {
    when(QualifierUtil.buildQualifierBeanMap(anyList())).thenReturn(Map.of("maven2", formatStoreManager));
    when(formatStoreManager.assetBlobStore("astrum")).thenReturn(assetBlobStore);

    AssetBlobRefMigrationTask underTest = new AssetBlobRefMigrationTask(List.of(formatStoreManager),
        READ_ASSETS_BATCH_SIZE);

    TaskConfiguration taskConfiguration = new TaskConfiguration();
    taskConfiguration.setString(FORMAT_FIELD_ID, "maven2");
    taskConfiguration.setString(CONTENT_STORE_FIELD_ID, "astrum");
    taskConfiguration.setId(UUID.randomUUID().toString());
    taskConfiguration.setTypeId(TYPE_ID);
    underTest.configure(taskConfiguration);

    underTest.execute();

    // Verify migration executed successfully
    verify(assetBlobStore, times(2)).updateBlobRefs(any());
  }

  private AssetBlobData newNotMigratedAssetBlobData() {
    AssetBlobData assetBlobData = mock(AssetBlobData.class);
    when(assetBlobData.blobRef()).thenReturn(mock(BlobRef.class));
    return assetBlobData;
  }
}
