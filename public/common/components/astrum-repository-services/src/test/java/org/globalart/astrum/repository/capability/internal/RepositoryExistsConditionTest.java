
package org.globalart.astrum.repository.capability.internal;

import java.util.Collections;
import java.util.function.Supplier;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryCreatedEvent;
import org.globalart.astrum.repository.manager.RepositoryDeletedEvent;
import org.globalart.astrum.repository.manager.RepositoryManager;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * {@link RepositoryExistsCondition} UTs.
 *
 * @since capabilities 2.0
 */
public class RepositoryExistsConditionTest
    extends EventManagerTestSupport
{

  static final String TEST_REPOSITORY = "test-repository";

  @Mock
  private Repository repository;

  @Mock
  private RepositoryManager repositoryManager;

  private RepositoryExistsCondition underTest;

  @Before
  public final void setUpRepositoryExistsCondition()
      throws Exception
  {
    when(repositoryManager.browse()).thenReturn(Collections.<Repository>emptyList());

    final Supplier<String> repositoryName = () -> TEST_REPOSITORY;

    when(repository.getName()).thenReturn(TEST_REPOSITORY);

    underTest = new RepositoryExistsCondition(eventManager, repositoryManager, repositoryName);
    underTest.bind();

    verify(eventManager).register(underTest);

    assertThat(underTest.isSatisfied(), is(false));

    underTest.handle(new RepositoryCreatedEvent(repository));
  }

  /**
   * Condition should be satisfied initially (because mocking done in setup).
   */
  @Test
  public void satisfiedWhenRepositoryExists() {
    assertThat(underTest.isSatisfied(), is(true));
  }

  /**
   * Condition should become satisfied and notification sent when repository is added.
   */
  @Test
  public void satisfiedWhenRepositoryAdded() {
    assertThat(underTest.isSatisfied(), is(true));

    underTest.handle(new RepositoryDeletedEvent(repository));
    underTest.handle(new RepositoryCreatedEvent(repository));
    assertThat(underTest.isSatisfied(), is(true));

    verifyEventManagerEvents(satisfied(underTest), unsatisfied(underTest), satisfied(underTest));
  }

  /**
   * Condition should become unsatisfied when repository is removed.
   */
  @Test
  public void repositoryIsRemoved() {
    assertThat(underTest.isSatisfied(), is(true));

    underTest.handle(new RepositoryDeletedEvent(repository));
    assertThat(underTest.isSatisfied(), is(false));

    verifyEventManagerEvents(satisfied(underTest), unsatisfied(underTest));
  }

  /**
   * Condition should remain satisfied when another repository is removed.
   */
  @Test
  public void noReactionWhenAnotherRepositoryIsRemoved() {
    assertThat(underTest.isSatisfied(), is(true));
    final Repository anotherRepository = mock(Repository.class);
    when(anotherRepository.getName()).thenReturn("another");
    underTest.handle(new RepositoryDeletedEvent(anotherRepository));
    assertThat(underTest.isSatisfied(), is(true));
  }

  /**
   * Event bus handler is removed when releasing.
   */
  @Test
  public void releaseRemovesItselfAsHandler() {
    underTest.release();

    verify(eventManager).unregister(underTest);
  }

}
