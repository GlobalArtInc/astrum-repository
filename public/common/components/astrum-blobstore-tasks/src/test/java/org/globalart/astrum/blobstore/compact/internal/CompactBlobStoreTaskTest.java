
package org.globalart.astrum.blobstore.compact.internal;

import java.time.Duration;
import java.time.OffsetDateTime;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.stream.Stream;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.api.BlobStoreUsageChecker;
import org.globalart.astrum.repository.move.ChangeRepositoryBlobStoreConfiguration;
import org.globalart.astrum.repository.move.ChangeRepositoryBlobStoreStore;
import org.globalart.astrum.scheduling.RecoveryModeService;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskUtils;

import org.apache.shiro.mgt.SecurityManager;
import org.apache.shiro.util.ThreadContext;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.doNothing;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.blobstore.common.BlobStoreParallelTaskSupport.ALL;
import static org.globalart.astrum.blobstore.common.BlobStoreParallelTaskSupport.BLOBSTORE_NAME_FIELD_ID;
import static org.globalart.astrum.blobstore.compact.internal.CompactBlobStoreTaskDescriptor.TYPE_ID;

@RunWith(MockitoJUnitRunner.Silent.class)
public class CompactBlobStoreTaskTest

{
  private final String BLOBSTORE_NAME = "test";

  private final String TASK_NAME = "test-task";

  @Mock
  ChangeRepositoryBlobStoreStore changeBlobstoreStore;

  @Mock
  BlobStoreUsageChecker blobStoreUsageChecker;

  @Mock
  TaskUtils taskUtils;

  @Mock
  RecoveryModeService recoveryModeService;

  @Mock
  BlobStoreManager blobStoreManager;

  @Mock
  SecurityManager securityManager;

  TaskConfiguration configuration;

  CompactBlobStoreTask underTest;

  @Before
  public void setUp() {
    ThreadContext.bind(securityManager);

    configuration = new TaskConfiguration();
    configuration.setString(BLOBSTORE_NAME_FIELD_ID, BLOBSTORE_NAME);
    configuration.setString(".name", TASK_NAME);
    configuration.setTypeId(TYPE_ID);
    configuration.setId(TASK_NAME);

    underTest =
        new CompactBlobStoreTask(changeBlobstoreStore, blobStoreUsageChecker, taskUtils, recoveryModeService, 5);
    underTest.install(blobStoreManager);
  }

  @After
  public void tearDown() {
    ThreadContext.unbindSecurityManager();
  }

  @Test
  public void checkForConflictsThrowsExceptionIfConflictingTaskIsRunning() {
    underTest.configure(configuration);

    BlobStore blobStore = mock(BlobStore.class);
    BlobStoreConfiguration config = mock(BlobStoreConfiguration.class);
    when(config.getName()).thenReturn(BLOBSTORE_NAME);
    when(blobStore.getBlobStoreConfiguration()).thenReturn(config);

    doThrow(new IllegalStateException("conflicting task"))
        .when(taskUtils)
        .checkForConflictingTasks(anyString(), anyString(), any(List.class), any(Map.class));
    when(changeBlobstoreStore.findByBlobStoreName(anyString())).thenReturn(Collections.emptyList());

    IllegalStateException exception =
        assertThrows(IllegalStateException.class, () -> underTest.checkForConflicts(BLOBSTORE_NAME));

    assertThat(exception.getMessage(), is("conflicting task"));
    verify(taskUtils, times(1)).checkForConflictingTasks(anyString(), anyString(), any(List.class), any(Map.class));
  }

  @Test
  public void checkForConflictsThrowsExceptionIfMoveTaskIsUnfinished() {
    ChangeRepositoryBlobStoreConfiguration record = getRecord("test", BLOBSTORE_NAME, "target-blobstore");

    underTest.configure(configuration);

    BlobStore blobStore = mock(BlobStore.class);
    BlobStoreConfiguration config = mock(BlobStoreConfiguration.class);
    when(config.getName()).thenReturn(BLOBSTORE_NAME);
    when(blobStore.getBlobStoreConfiguration()).thenReturn(config);

    doNothing()
        .when(taskUtils)
        .checkForConflictingTasks(anyString(), anyString(), any(List.class), any(Map.class));
    when(changeBlobstoreStore.findByBlobStoreName(anyString())).thenReturn(Collections.singletonList(record));

    IllegalStateException exception =
        assertThrows(IllegalStateException.class, () -> underTest.checkForConflicts(BLOBSTORE_NAME));

    assertThat(exception.getMessage(),
        is(String.format("found unfinished move task(s) using blobstore '%s', task can't be executed",
            BLOBSTORE_NAME)));
    verify(taskUtils, times(1)).checkForConflictingTasks(anyString(), anyString(), any(List.class), any(Map.class));
    verify(changeBlobstoreStore, times(1)).findByBlobStoreName(eq(BLOBSTORE_NAME));
  }

  @Test
  public void testAllBlobStores() throws Exception {
    configuration.setString(BLOBSTORE_NAME_FIELD_ID, ALL);

    List<BlobStore> blobStores = runCompact("blobstore-one", "blobstore-two");

    assertThat("All blob stores should be processed", underTest.result(), is(2));
    verify(blobStores.get(0), times(1)).compact(any(BlobStoreUsageChecker.class), any(Duration.class));
    verify(blobStores.get(1), times(1)).compact(any(BlobStoreUsageChecker.class), any(Duration.class));
  }

  @Test
  public void testSingleBlobStore() throws Exception {
    configuration.setString(BLOBSTORE_NAME_FIELD_ID, "blobstore-one");

    List<BlobStore> blobStores = runCompact("blobstore-one", "blobstore-two");

    assertThat("Only specified blob store should be processed", underTest.result(), is(1));
    verify(blobStores.get(0), times(1)).compact(any(BlobStoreUsageChecker.class), any(Duration.class));
    verify(blobStores.get(1), times(0)).compact(any(BlobStoreUsageChecker.class), any(Duration.class));
  }

  @Test
  public void testMultipleBlobStores() throws Exception {
    configuration.setString(BLOBSTORE_NAME_FIELD_ID, "blobstore-one,blobstore-three");

    List<BlobStore> blobStores = runCompact("blobstore-one", "blobstore-two", "blobstore-three");

    assertThat("Multiple specified blob stores should be processed", underTest.result(), is(2));
    verify(blobStores.get(0), times(1)).compact(any(BlobStoreUsageChecker.class), any(Duration.class));
    verify(blobStores.get(1), times(0)).compact(any(BlobStoreUsageChecker.class), any(Duration.class));
    verify(blobStores.get(2), times(1)).compact(any(BlobStoreUsageChecker.class), any(Duration.class));
  }

  @Test
  public void testGetMessageForAllBlobStores() {
    configuration.setString(BLOBSTORE_NAME_FIELD_ID, ALL);
    underTest.configure(configuration);

    assertThat(underTest.getMessage(), is("Compacting all blob stores"));
  }

  @Test
  public void testGetMessageForSingleBlobStore() {
    configuration.setString(BLOBSTORE_NAME_FIELD_ID, "my-blobstore");
    underTest.configure(configuration);

    assertThat(underTest.getMessage(), is("Compacting [my-blobstore]"));
  }

  @Test
  public void testResultReturnsProcessedCount() throws Exception {
    configuration.setString(BLOBSTORE_NAME_FIELD_ID, "blobstore-one,blobstore-two");

    List<BlobStore> blobStores = runCompact("blobstore-one", "blobstore-two");

    assertThat("Result should return number of processed blob stores", underTest.result(), is(2));
    verify(blobStores.get(0), times(1)).compact(any(BlobStoreUsageChecker.class), any(Duration.class));
    verify(blobStores.get(1), times(1)).compact(any(BlobStoreUsageChecker.class), any(Duration.class));
  }

  @Test
  public void testCompact_failsWhenRecoveryModeActive() throws Exception {
    configuration.setString(BLOBSTORE_NAME_FIELD_ID, "blobstore-one");
    doThrow(
        new IllegalStateException("expected exception"))
            .when(recoveryModeService)
            .ensureNotInRecoveryMode(any());

    List<BlobStore> blobStores = runCompact("blobstore-one");

    verify(recoveryModeService).ensureNotInRecoveryMode(any());
    verify(blobStores.get(0), times(0)).compact(any(BlobStoreUsageChecker.class), any(Duration.class));
    assertThat("No blob stores should be processed when recovery mode is active", underTest.result(), is(0));
  }

  @Test
  public void testCompact_taskFailsWhenBlobStoreCompactThrows() {
    configuration.setString(BLOBSTORE_NAME_FIELD_ID, "blobstore-one");

    List<BlobStore> blobStores = mockBlobStores("blobstore-one");
    doThrow(new RuntimeException("compaction failed"))
        .when(blobStores.get(0))
        .compact(any(BlobStoreUsageChecker.class), any(Duration.class));
    doNothing()
        .when(taskUtils)
        .checkForConflictingTasks(anyString(), anyString(), any(List.class), any(Map.class));
    when(changeBlobstoreStore.findByBlobStoreName(anyString())).thenReturn(Collections.emptyList());

    underTest.configure(configuration);

    assertThrows(Exception.class, () -> underTest.call());

    verify(blobStores.get(0), times(1)).compact(any(BlobStoreUsageChecker.class), any(Duration.class));
    assertThat("Failed blob store must not be counted as processed", underTest.result(), is(0));
  }

  @Test
  public void testCheckForConflicts_withNullRecoveryModeService() {
    // Create task with null RecoveryModeService (CORE edition scenario)
    CompactBlobStoreTask taskWithNullRecoveryMode =
        new CompactBlobStoreTask(changeBlobstoreStore, blobStoreUsageChecker, taskUtils, null, 5);
    taskWithNullRecoveryMode.install(blobStoreManager);
    taskWithNullRecoveryMode.configure(configuration);

    doNothing()
        .when(taskUtils)
        .checkForConflictingTasks(anyString(), anyString(), any(List.class), any(Map.class));
    when(changeBlobstoreStore.findByBlobStoreName(anyString())).thenReturn(Collections.emptyList());

    // checkForConflicts should complete without throwing NullPointerException
    taskWithNullRecoveryMode.checkForConflicts(BLOBSTORE_NAME);

    // Verify that taskUtils.checkForConflictingTasks was still called
    verify(taskUtils, times(1)).checkForConflictingTasks(anyString(), anyString(), any(List.class), any(Map.class));
    verify(changeBlobstoreStore, times(1)).findByBlobStoreName(eq(BLOBSTORE_NAME));
  }

  private List<BlobStore> runCompact(final String... names) throws Exception {
    List<BlobStore> blobStores = mockBlobStores(names);

    // Mock compact behavior
    for (BlobStore blobStore : blobStores) {
      doNothing().when(blobStore).compact(any(BlobStoreUsageChecker.class), any(Duration.class));
    }

    doNothing()
        .when(taskUtils)
        .checkForConflictingTasks(anyString(), anyString(), any(List.class), any(Map.class));
    when(changeBlobstoreStore.findByBlobStoreName(anyString())).thenReturn(Collections.emptyList());

    underTest.configure(configuration);
    underTest.call();

    return blobStores;
  }

  private List<BlobStore> mockBlobStores(final String... names) {
    List<BlobStore> blobStores = Stream.of(names)
        .map(name -> {
          BlobStore blobStore = mock(BlobStore.class);
          BlobStoreConfiguration config = mock(BlobStoreConfiguration.class);
          lenient().when(config.getName()).thenReturn(name);
          lenient().when(blobStore.getBlobStoreConfiguration()).thenReturn(config);
          lenient().when(blobStoreManager.get(name)).thenReturn(blobStore);
          return blobStore;
        })
        .toList();

    lenient().when(blobStoreManager.browse()).thenReturn(blobStores);

    return blobStores;
  }

  private ChangeRepositoryBlobStoreConfiguration getRecord(
      final String name,
      final String sourceBlobStoreName,
      final String targetBlobStoreName)
  {
    return new ChangeRepositoryBlobStoreConfiguration()
    {
      @Override
      public String getName() {
        return name;
      }

      @Override
      public void setName(final String name) {

      }

      @Override
      public String getTargetBlobStoreName() {
        return targetBlobStoreName;
      }

      @Override
      public void setTargetBlobStoreName(final String targetBlobStoreName) {

      }

      @Override
      public String getSourceBlobStoreName() {
        return sourceBlobStoreName;
      }

      @Override
      public void setSourceBlobStoreName(final String sourceBlobStoreName) {

      }

      @Override
      public OffsetDateTime getStarted() {
        return null;
      }

      @Override
      public void setStarted(final OffsetDateTime processStartDate) {

      }
    };
  }
}
