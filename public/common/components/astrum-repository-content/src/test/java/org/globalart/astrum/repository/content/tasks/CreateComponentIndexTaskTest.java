
package org.globalart.astrum.repository.content.tasks;

import org.globalart.astrum.scheduling.TaskInterruptedException;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;

@RunWith(MockitoJUnitRunner.Silent.class)
public class CreateComponentIndexTaskTest
{
  @Mock
  private CreateComponentIndexService createComponentIndexService;

  @Test
  public void testGetMessage() {
    CreateComponentIndexTask underTest = new CreateComponentIndexTask(createComponentIndexService, false);
    assertThat(underTest.getMessage(), containsString("indexes"));
  }

  @Test
  public void testExecuteCallsService() throws Exception {
    CreateComponentIndexTask underTest = new CreateComponentIndexTask(createComponentIndexService, false);
    Object result = underTest.execute();
    assertThat(result, is(nullValue()));
    verify(createComponentIndexService).recreateComponentIndexes();
  }

  @Test(expected = TaskInterruptedException.class)
  public void testExecuteThrowsWhenDisabled() throws Exception {
    CreateComponentIndexTask underTest = new CreateComponentIndexTask(createComponentIndexService, true);
    underTest.execute();
  }

  @Test
  public void testExecuteDoesNotCallServiceWhenDisabled() throws Exception {
    CreateComponentIndexTask underTest = new CreateComponentIndexTask(createComponentIndexService, true);
    try {
      underTest.execute();
    }
    catch (TaskInterruptedException expected) {
      // expected
    }
    verifyNoInteractions(createComponentIndexService);
  }

  @Test(expected = NullPointerException.class)
  public void testNullServiceRejected() {
    new CreateComponentIndexTask(null, false);
  }
}
