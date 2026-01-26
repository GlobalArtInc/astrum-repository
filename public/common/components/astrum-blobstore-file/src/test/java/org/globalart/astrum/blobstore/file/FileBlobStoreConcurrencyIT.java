
package org.globalart.astrum.blobstore.file;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Path;
import java.security.NoSuchAlgorithmException;
import java.time.Duration;
import java.util.HashSet;
import java.util.Queue;
import java.util.Random;
import java.util.Set;
import java.util.UUID;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.Callable;
import java.util.concurrent.ConcurrentLinkedDeque;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import org.globalart.astrum.blobstore.BlobStoreReconciliationLogger;
import org.globalart.astrum.blobstore.DefaultBlobIdLocationResolver;
import org.globalart.astrum.blobstore.MetricsInputStream;
import org.globalart.astrum.blobstore.MockBlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobMetrics;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreException;
import org.globalart.astrum.blobstore.file.internal.FileOperations;
import org.globalart.astrum.blobstore.file.internal.SimpleFileOperations;
import org.globalart.astrum.blobstore.file.internal.datastore.metrics.DatastoreFileBlobStoreMetricsService;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaService;
import org.globalart.astrum.blobstore.quota.BlobStoreQuotaUsageChecker;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.log.DryRunPrefix;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.bootstrap.entrypoint.configuration.DirectoryHelper;
import org.globalart.astrum.scheduling.internal.PeriodicJobServiceImpl;

import com.google.common.base.Objects;
import com.google.common.collect.ImmutableMap;
import com.google.common.io.ByteStreams;
import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkState;
import static com.google.common.io.ByteStreams.nullOutputStream;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.atLeastOnce;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.blobstore.api.BlobStore.BLOB_NAME_HEADER;
import static org.globalart.astrum.blobstore.api.BlobStore.CREATED_BY_HEADER;

/**
 * {@link FileBlobStore} concurrency tests.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class FileBlobStoreConcurrencyIT
{
  private static final Logger log = LoggerFactory.getLogger(FileBlobStoreConcurrencyIT.class);

  @Rule
  public TemporaryFolder tmpDir = new TemporaryFolder();

  public static final ImmutableMap<String, String> TEST_HEADERS = ImmutableMap.of(
      CREATED_BY_HEADER, "test",
      BLOB_NAME_HEADER, "test/randomData.bin");

  public static final int BLOB_MAX_SIZE_BYTES = 5_000_000;

  private static final int QUOTA_CHECK_INTERVAL = 1;

  private FileBlobStore underTest;

  @Mock
  private DatastoreFileBlobStoreMetricsService metricsStore;

  private BlobStoreQuotaUsageChecker blobStoreQuotaUsageChecker;

  @Mock
  private FileBlobDeletionIndex fileBlobDeletionIndex;

  @Mock
  NodeAccess nodeAccess;

  @Mock
  DryRunPrefix dryRunPrefix;

  @Mock
  BlobStoreQuotaService quotaService;

  @Mock
  FileOperations fileOperations;

  @Mock
  private BlobStoreReconciliationLogger reconciliationLogger;

  private final DirectoryHelper directoryHelper = new DirectoryHelper();

  @Before
  public void setUp() throws Exception {
    Path root = tmpDir.newFolder().toPath();
    Path content = root.resolve("content");

    when(nodeAccess.getId()).thenReturn(UUID.randomUUID().toString());
    when(dryRunPrefix.get()).thenReturn("");

    ApplicationDirectories applicationDirectories = mock(ApplicationDirectories.class);
    when(applicationDirectories.getWorkDirectory(anyString())).thenReturn(root.toFile());

    final BlobStoreConfiguration config = new MockBlobStoreConfiguration();
    config.attributes(FileBlobStore.CONFIG_KEY).set(FileBlobStore.PATH_KEY, root.toString());

    blobStoreQuotaUsageChecker = spy(
        new BlobStoreQuotaUsageChecker(new PeriodicJobServiceImpl(10), QUOTA_CHECK_INTERVAL, quotaService));

    this.underTest = new FileBlobStore(content, new DefaultBlobIdLocationResolver(),
        new SimpleFileOperations(directoryHelper), applicationDirectories, directoryHelper, metricsStore, config,
        nodeAccess, dryRunPrefix, reconciliationLogger, 0L, blobStoreQuotaUsageChecker, fileBlobDeletionIndex);
    underTest.start();
  }

  @After
  public void tearDown() throws Exception {
    if (underTest != null) {
      underTest.stop();
    }
  }

  @Test
  public void concurrencyTest() throws Exception {

    final Random random = new Random();

    int numberOfCreators = 10;
    int numberOfDeleters = 5;
    int numberOfReaders = 30;
    int numberOfCompactors = 1;
    int numberOfShufflers = 3;

    final Queue<BlobId> blobIdsInTheStore = new ConcurrentLinkedDeque<>();

    final Set<BlobId> deletedIds = new HashSet<>();

    int numberOfIterations = 15;
    int timeoutMinutes = 5;
    final ConcurrentRunner runner = new ConcurrentRunner(numberOfIterations, timeoutMinutes * 60L);

    runner.addTask(numberOfCreators, () -> {
      final byte[] data = new byte[random.nextInt(BLOB_MAX_SIZE_BYTES) + 1];
      random.nextBytes(data);
      final Blob blob = underTest.create(new ByteArrayInputStream(data), TEST_HEADERS);

      blobIdsInTheStore.add(blob.getId());
    });

    runner.addTask(numberOfReaders, () -> {
      final BlobId blobId = blobIdsInTheStore.peek();

      log.info("Attempting to read " + blobId);

      if (blobId == null) {
        return;
      }

      final Blob blob = underTest.get(blobId);
      if (blob == null) {
        log.info("Attempted to obtain blob, but it was deleted:" + blobId);
        return;
      }

      try (InputStream inputStream = blob.getInputStream()) {
        readContentAndValidateMetrics(blobId, inputStream, blob.getMetrics());
      }
      catch (BlobStoreException e) {
        checkState(deletedIds.contains(e.getBlobId()));
        // This is normal operation if another thread deletes your blob after you obtain a Blob reference
        log.warn("Concurrent deletion suspected while calling blob.getInputStream().", e);
      }
    });

    runner.addTask(numberOfDeleters, () -> {
      final BlobId blobId = blobIdsInTheStore.poll();
      if (blobId == null) {
        log.info("deleter: null blob id");
        return;
      }
      log.info("Deleting {}", blobId);

      // There's a race condition here, we need to note that we're attempting to delete this before the deletion
      // goes through, otherwise we may fail the check, above.
      deletedIds.add(blobId);
      underTest.delete(blobId, "Testing concurrency");
    });

    // Shufflers pull blob IDs off the front of the queue and stick them on the back, to make the blobID queue a bit
    // less orderly
    runner.addTask(numberOfShufflers, () -> {
      final BlobId blobId = blobIdsInTheStore.poll();
      if (blobId != null) {
        blobIdsInTheStore.add(blobId);
      }
    });

    runner.addTask(numberOfCompactors, () -> underTest.compact(null, Duration.ofDays(100)));

    runner.go();

    verify(metricsStore).init(underTest);
    verify(quotaService, atLeastOnce()).checkQuota(underTest);
  }

  /**
   * Read all the content from a blob, and compare it with the metrics on file in the blob store.
   *
   * @throws RuntimeException if there is any deviation
   */
  private void readContentAndValidateMetrics(
      final BlobId blobId,
      final InputStream inputStream,
      final BlobMetrics metadataMetrics) throws NoSuchAlgorithmException, IOException
  {
    final MetricsInputStream measured = new MetricsInputStream(inputStream);
    ByteStreams.copy(measured, nullOutputStream());

    checkEqual("stream length", metadataMetrics.getContentSize(), measured.getSize(), blobId);
    checkEqual("SHA1 hash", metadataMetrics.getSha1Hash(), measured.getMessageDigest(), blobId);
  }

  private void checkEqual(
      final String propertyName,
      final Object expected,
      final Object measured,
      final BlobId blobId)
  {
    if (!Objects.equal(measured, expected)) {
      throw new RuntimeException(
          "Blob " + blobId + "'s measured " + propertyName + " differed from its metadata. Expected " + expected +
              " but was " + measured + ".");
    }
  }

  @FunctionalInterface
  private interface ConcurrentTask
  {
    void run() throws Exception;
  }

  private static class ConcurrentRunner
  {
    private final int iterations;

    private final long timeoutSeconds;

    private final List<ConcurrentTask> allTasks = new ArrayList<>();

    ConcurrentRunner(final int iterations, final long timeoutSeconds) {
      this.iterations = iterations;
      this.timeoutSeconds = timeoutSeconds;
    }

    void addTask(final int threads, final ConcurrentTask task) {
      for (int i = 0; i < threads; i++) {
        allTasks.add(task);
      }
    }

    void go() throws Exception {
      ExecutorService executor = Executors.newFixedThreadPool(allTasks.size());
      try {
        for (int i = 0; i < iterations; i++) {
          List<Callable<Void>> callables = new ArrayList<>();
          for (ConcurrentTask task : allTasks) {
            callables.add(() -> {
              task.run();
              return null;
            });
          }
          List<Future<Void>> futures = executor.invokeAll(callables, timeoutSeconds, TimeUnit.SECONDS);
          for (Future<Void> f : futures) {
            f.get();
          }
        }
      }
      finally {
        executor.shutdownNow();
      }
    }
  }
}
