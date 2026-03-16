
package org.globalart.astrum.repository.search.sql.index;

import java.util.Optional;

import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.common.scheduling.PeriodicJobService.PeriodicJob;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.content.search.SearchFacet;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.search.sql.store.SearchStore;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyCollection;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.timeout;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link SqlSearchEventHandler#requestIndex(String, int, Repository)}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class SqlSearchEventHandlerTest
{
  private static final String FORMAT = "raw";

  @Mock
  private SearchStore searchStore;

  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private PeriodicJobService periodicJobService;

  @Mock
  private Repository repository;

  @Mock
  private Type proxyType;

  @Mock
  private SearchFacet searchFacet;

  private SqlSearchEventHandler underTest;

  @Before
  public void setUp() throws Exception {
    PeriodicJob periodicJob = mock(PeriodicJob.class);
    when(periodicJobService.schedule(any(Runnable.class), anyInt())).thenReturn(periodicJob);
    when(repository.optionalFacet(SearchFacet.class)).thenReturn(Optional.of(searchFacet));
    when(proxyType.getValue()).thenReturn(ProxyType.NAME);
    when(repository.getType()).thenReturn(proxyType);

    underTest = new SqlSearchEventHandler(
        searchStore, repositoryManager, periodicJobService, /* flushOnCount */ 100, /* flushOnSeconds */ 2,
        /* noPurgeDelay */ true, /* poolSize */ 1);
    underTest.start();
  }

  @After
  public void tearDown() throws Exception {
    if (underTest != null) {
      underTest.stop();
    }
  }

  @Test
  public void requestIndexWritesThroughSearchFacetViaThreadPool() {
    underTest.requestIndex(FORMAT, 42, repository);

    verify(searchFacet, timeout(2000)).index(any());
    assertThat("index not routed through pendingRequests batch path",
        underTest.isCalmPeriod(), is(true));
  }

  @Test
  public void requestIndexIgnoresNonPositiveComponentId() {
    underTest.requestIndex(FORMAT, 0, repository);
    underTest.requestIndex(FORMAT, -1, repository);

    verify(searchFacet, never()).index(anyCollection());
  }

  @Test
  public void requestIndexShortCircuitsWhenProcessingDisabled() {
    underTest.setProcessEvents(false);

    underTest.requestIndex(FORMAT, 42, repository);

    verify(searchFacet, never()).index(anyCollection());
  }

  @Test
  public void requestIndexDoesNothingWhenSearchFacetMissing() {
    when(repository.optionalFacet(SearchFacet.class)).thenReturn(Optional.empty());

    underTest.requestIndex(FORMAT, 42, repository);

    verify(searchFacet, never()).index(anyCollection());
  }

  @Test
  public void requestIndexUsesBatchPathForHostedRepository() {
    Repository hostedRepo = mock(Repository.class);
    Type hostedType = mock(Type.class);
    when(hostedType.getValue()).thenReturn(HostedType.NAME);
    when(hostedRepo.getType()).thenReturn(hostedType);
    when(hostedRepo.optionalFacet(SearchFacet.class)).thenReturn(Optional.of(searchFacet));

    underTest.requestIndex(FORMAT, 42, hostedRepo);

    verify(searchFacet, never()).index(anyCollection());
    assertThat("hosted request should enter batch path (pendingRequests not empty)",
        underTest.isCalmPeriod(), is(false));
  }
}
