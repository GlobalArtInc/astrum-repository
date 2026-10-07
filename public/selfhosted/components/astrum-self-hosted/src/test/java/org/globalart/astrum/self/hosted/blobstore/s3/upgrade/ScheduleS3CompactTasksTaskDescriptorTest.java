
package org.globalart.astrum.self.hosted.blobstore.s3.upgrade;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

public class ScheduleS3CompactTasksTaskDescriptorTest
{
  private ScheduleS3CompactTasksTaskDescriptor underTest;

  @Test
  public void testNotExposedByDefault() {
    underTest = new ScheduleS3CompactTasksTaskDescriptor(false);

    assertThat("Task should not be exposed by default", underTest.isExposed(), is(false));
    assertThat("Task should not be visible by default", underTest.isVisible(), is(false));
  }

  @Test
  public void testVisibleAndExposedWhenEnabled() {
    underTest = new ScheduleS3CompactTasksTaskDescriptor(true);

    assertThat("Task should be visible when enabled", underTest.isVisible(), is(true));
    assertThat("Task should be exposed when enabled", underTest.isExposed(), is(true));
  }

  @Test
  public void testTaskType() {
    underTest = new ScheduleS3CompactTasksTaskDescriptor(false);

    assertThat("Task type should be ScheduleS3CompactTasksTask",
        underTest.getType(),
        equalTo(ScheduleS3CompactTasksTask.class));
  }

  @Test
  public void testName() {
    underTest = new ScheduleS3CompactTasksTaskDescriptor(false);

    assertThat("Task name should be set", underTest.getName(), notNullValue());
    assertThat("Task name should describe the task",
        underTest.getName(),
        equalTo("Schedule compact blobstore tasks based on expiration policy"));
  }

  @Test
  public void testIsVisibleReturnsFalseWhenNotExposed() {
    underTest = new ScheduleS3CompactTasksTaskDescriptor(false);

    assertThat("Task should not be visible in UI when exposed is false",
        underTest.isVisible(),
        is(false));
  }

  @Test
  public void testIsExposedReturnsFalseWhenNotExposed() {
    underTest = new ScheduleS3CompactTasksTaskDescriptor(false);

    assertThat("Task should not be exposed when exposed is false",
        underTest.isExposed(),
        is(false));
  }
}
