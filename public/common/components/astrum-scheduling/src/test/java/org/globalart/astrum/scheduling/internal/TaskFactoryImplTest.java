
package org.globalart.astrum.scheduling.internal;

import java.util.Iterator;
import java.util.List;
import java.util.UUID;

import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.scheduling.Task;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskDescriptor;
import org.globalart.astrum.scheduling.TaskInfo;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.springframework.beans.factory.ObjectProvider;
import org.springframework.context.ApplicationContext;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link TaskFactoryImpl}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class TaskFactoryImplTest

{
  @Mock
  private ApplicationContext applicationContext;

  @Mock
  private TaskInfo taskInfo;

  @Mock
  private DatabaseCheck databaseCheck;

  @Mock
  private ObjectProvider<SimpleTask> taskProvider;

  private TaskFactoryImpl underTest;

  @Before
  public void setUp() {
    when(taskProvider.getIfUnique()).thenReturn(new SimpleTask());
    when(applicationContext.getBeanProvider(SimpleTask.class)).thenReturn(taskProvider);

    when(databaseCheck.isAllowedByVersion(any())).thenReturn(true);
    underTest = new TaskFactoryImpl(databaseCheck, List.of(), applicationContext);
  }

  /*
   * register and find descriptor
   */
  @Test
  public void testRegisterAndFindDescriptor() {
    assert underTest.getDescriptors().isEmpty();

    underTest.addDescriptor(new SimpleTaskDescriptor());
    assertThat(underTest.getDescriptors(), hasSize(1));

    assertThat(underTest.findDescriptor(SimpleTaskDescriptor.TYPE_ID), notNullValue());

    assertThat(underTest.findDescriptor("no-such-type-id"), nullValue());
  }

  /*
   * descriptor list is immutable
   */
  @Test
  public void testDescriptorListIsImmutable() {
    SimpleTaskDescriptor descriptor = new SimpleTaskDescriptor();
    List<TaskDescriptor> descriptors = underTest.getDescriptors();
    // can not add directly to the list
    assertThrows(UnsupportedOperationException.class, () -> descriptors.add(descriptor));

    // can not remove from list
    underTest.addDescriptor(new SimpleTaskDescriptor());

    Iterator<TaskDescriptor> iter = underTest.getDescriptors().iterator();
    assertThrows(UnsupportedOperationException.class, () -> iter.remove());
  }

  /*
   * create missing descriptor
   */
  @Test
  public void testCreateMissingDescriptor() {
    TaskConfiguration config = new TaskConfiguration();
    config.setId(UUID.randomUUID().toString());
    config.setTypeId("no-such-type-id");

    assertThrows(IllegalArgumentException.class, () -> underTest.create(config, taskInfo));
  }

  @Test
  public void testCreateTask() {
    underTest.addDescriptor(new SimpleTaskDescriptor());

    TaskConfiguration config = new TaskConfiguration();
    config.setId(UUID.randomUUID().toString());
    config.setTypeId(SimpleTaskDescriptor.TYPE_ID);
    Task task = underTest.create(config, taskInfo);

    assertThat(task, notNullValue());
    assertThat(task, instanceOf(SimpleTask.class));
    assertThat(task.getTaskInfo(), is(taskInfo));
  }
}
