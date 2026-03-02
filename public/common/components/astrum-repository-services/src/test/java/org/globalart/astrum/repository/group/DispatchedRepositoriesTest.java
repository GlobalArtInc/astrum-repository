
package org.globalart.astrum.repository.group;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.group.GroupHandler.DispatchedRepositories;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.CoreMatchers.containsString;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class DispatchedRepositoriesTest
{
  private static final String REPOSITORY_1 = "repository1";

  private static final String REPOSITORY_2 = "repository2";

  private static final String REPOSITORY_3 = "repository3";

  @Mock
  private Repository repository1;

  @Mock
  private Repository repository2;

  @Mock
  private Repository repository3;

  private DispatchedRepositories underTest;

  @Before
  public void setUp() throws Exception {
    underTest = new DispatchedRepositories();
    when(repository1.toString()).thenReturn(REPOSITORY_1);
    when(repository1.getName()).thenReturn(REPOSITORY_1);
    when(repository2.toString()).thenReturn(REPOSITORY_2);
    when(repository2.getName()).thenReturn(REPOSITORY_2);
    when(repository3.toString()).thenReturn(REPOSITORY_3);
    when(repository3.getName()).thenReturn(REPOSITORY_3);
  }

  @Test
  public void checkDispatchedRepositoryInsertionWillPreserveOrder() {
    underTest.add(repository1);
    underTest.add(repository2);
    underTest.add(repository3);

    assertThat(underTest.getDispatched().toString(),
        containsString(String.format("[%s, %s, %s]", REPOSITORY_1, REPOSITORY_2, REPOSITORY_3)));
  }

  @Test
  public void checkDispatchedRepositoryInsertionWillPreserveOrderWhenAlternateSequence() {
    underTest.add(repository3);
    underTest.add(repository1);
    underTest.add(repository2);

    assertThat(underTest.getDispatched().toString(),
        containsString(String.format("[%s, %s, %s]", REPOSITORY_3, REPOSITORY_1, REPOSITORY_2)));
  }
}
