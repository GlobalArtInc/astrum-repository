
package org.globalart.astrum.repository.content.browse;

import java.util.Optional;

import org.globalart.astrum.repository.Repository;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class TrimBrowseNodesTaskTest
{
  @Mock
  private Repository repository;

  @Mock
  private BrowseFacet browseFacet;

  private TrimBrowseNodesTask underTest;

  @Before
  public void setUp() {
    underTest = new TrimBrowseNodesTask();
  }

  @Test
  public void testAppliesToRepositoryWithBrowseFacet() {
    when(repository.optionalFacet(BrowseFacet.class)).thenReturn(Optional.of(browseFacet));

    assertThat(underTest.appliesTo(repository), is(true));
  }

  @Test
  public void testDoesNotApplyToRepositoryWithoutBrowseFacet() {
    when(repository.optionalFacet(BrowseFacet.class)).thenReturn(Optional.empty());

    assertThat(underTest.appliesTo(repository), is(false));
  }

  @Test
  public void testDoesNotApplyToNullRepository() {
    assertThat(underTest.appliesTo(null), is(false));
  }
}
