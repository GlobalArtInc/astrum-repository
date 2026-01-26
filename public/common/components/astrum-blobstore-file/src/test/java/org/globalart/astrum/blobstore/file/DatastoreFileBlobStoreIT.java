
package org.globalart.astrum.blobstore.file;

import java.time.Duration;

import org.globalart.astrum.blobstore.file.internal.datastore.DatastoreFileBlobDeletionIndex;
import org.globalart.astrum.blobstore.internal.softdeleted.SoftDeletedBlobsDAO;
import org.globalart.astrum.blobstore.internal.softdeleted.SoftDeletedBlobsStoreImpl;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Before;
import org.junit.Rule;
import org.mockito.Mock;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.Mockito.doAnswer;

/**
 * {@link FileBlobStore} integration tests.
 */
public class DatastoreFileBlobStoreIT
    extends FileBlobStoreITSupport
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(SoftDeletedBlobsDAO.class);

  @Mock
  private EventManager eventManager;

  @Mock
  private PeriodicJobService periodicJobService;

  private SoftDeletedBlobsStoreImpl store;

  @Before
  public void setupPeriodicJobService() {
    // Setup synchronous running of the soft deleted blob index for the tests
    doAnswer(invocation -> {
      invocation.getArgument(0, Runnable.class).run();
      return null;
    })
        .when(periodicJobService)
        .runOnce(any(Runnable.class), anyInt());
  }

  @Override
  protected FileBlobDeletionIndex fileBlobDeletionIndex() {
    if (store == null) {
      store = new SoftDeletedBlobsStoreImpl(sessionRule);
      store.setDependencies(eventManager);
    }

    return new DatastoreFileBlobDeletionIndex(store, periodicJobService, Duration.ofSeconds(1));
  }

}
