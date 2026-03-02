
package org.globalart.astrum.repository.internal.search.index.task;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.search.index.SearchIndexFacet;
import org.globalart.astrum.repository.search.index.SearchUpdateService;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskScheduler;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SearchUpdateTaskTest
{
  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private SearchUpdateService searchUpdateService;

  @Mock
  private Repository repository1;

  @Mock
  private Repository repository2;

  @Mock
  private SearchIndexFacet searchIndexFacet1;

  @Mock
  private SearchIndexFacet searchIndexFacet2;

  @Mock
  private TaskScheduler taskScheduler;

  private final TaskConfiguration configuration = new TaskConfiguration();

  private SearchUpdateTask underTest;

  @Before
  public void setup() {
    when(repository1.getName()).thenReturn("repository1");
    when(repository2.getName()).thenReturn("repository1");
    when(repository1.facet(SearchIndexFacet.class)).thenReturn(searchIndexFacet1);
    when(repository2.facet(SearchIndexFacet.class)).thenReturn(searchIndexFacet2);
    when(repositoryManager.get("repository1")).thenReturn(repository1);
    when(repositoryManager.get("repository2")).thenReturn(repository2);

    configuration.setId("test");
    configuration.setTypeId("test");

    underTest = new SearchUpdateTask(repositoryManager, searchUpdateService, taskScheduler);
  }

  @Test
  public void runOnOneRepository() {
    configuration.setString("repositoryNames", "repository1");
    underTest.configure(configuration);
    underTest.execute();
    verify(searchIndexFacet1).rebuildIndex();
    verify(searchUpdateService).doneReindexing(repository1);
  }

  @Test
  public void runOnMultipleRepositories() {
    configuration.setString("repositoryNames", "repository1,repository2");
    underTest.configure(configuration);
    underTest.execute();
    verify(searchIndexFacet1).rebuildIndex();
    verify(searchIndexFacet2).rebuildIndex();
    verify(searchUpdateService).doneReindexing(repository1);
    verify(searchUpdateService).doneReindexing(repository2);
  }

  @Test
  public void unknownRepository() {
    configuration.setString("repositoryNames", "repository1,unknown,repository2");
    underTest.configure(configuration);
    underTest.execute();
    verify(searchIndexFacet1).rebuildIndex();
    verify(searchIndexFacet2).rebuildIndex();
    verify(searchUpdateService).doneReindexing(repository1);
    verify(searchUpdateService).doneReindexing(repository2);
  }

  @Test
  public void runOnMultipleRepositoriesButFailRebuildingIndex() {
    doThrow(new RuntimeException("failed to process cluster event (delete-index)"))
        .when(searchIndexFacet1)
        .rebuildIndex();
    configuration.setString("repositoryNames", "repository1,repository2");
    underTest.configure(configuration);
    underTest.execute();
    verify(searchIndexFacet2).rebuildIndex();
    verify(searchUpdateService).doneReindexing(repository2);
  }
}
