
package org.globalart.astrum.repository.content.browse;

import java.util.Optional;
import java.util.function.Consumer;

import org.globalart.astrum.repository.Repository;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class RebuildBrowseNodeServiceImplTest
{
  @Mock
  private Repository repository;

  @Mock
  private BrowseFacet browseFacet;

  private RebuildBrowseNodeServiceImpl underTest;

  @Before
  public void setUp() {
    underTest = new RebuildBrowseNodeServiceImpl();
  }

  @Test
  public void testRebuildDelegatesToBrowseFacet() throws Exception {
    when(repository.optionalFacet(BrowseFacet.class)).thenReturn(Optional.of(browseFacet));

    Consumer<String> progressUpdater = msg -> {
    };
    underTest.rebuild(repository, progressUpdater);

    verify(browseFacet).rebuildBrowseNodes(progressUpdater);
  }

  @Test
  public void testRebuildDoesNothingWhenNoBrowseFacet() throws Exception {
    when(repository.optionalFacet(BrowseFacet.class)).thenReturn(Optional.empty());

    underTest.rebuild(repository, msg -> {
    });

    verify(browseFacet, never()).rebuildBrowseNodes(any());
  }
}
