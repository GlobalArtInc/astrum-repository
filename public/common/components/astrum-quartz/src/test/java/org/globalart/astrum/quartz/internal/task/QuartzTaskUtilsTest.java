
package org.globalart.astrum.quartz.internal.task;

import org.globalart.astrum.scheduling.TaskConfiguration;

import com.google.common.collect.ImmutableMap;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;
import org.quartz.JobDataMap;
import org.quartz.JobDetail;

import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.scheduling.TaskConfiguration.REMOVE_ATTRIBUTE_MARKER;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class QuartzTaskUtilsTest
{
  public static final String FOO_KEY = "foo";

  public static final String BAR_KEY = "bar";

  public static final String FOO_VALUE = "fooValue";

  @Mock
  private JobDetail jobDetail;

  @Mock
  private JobDataMap jobDataMap;

  @Mock
  private TaskConfiguration taskConfiguration;

  @Before
  public void setup() {
    when(jobDetail.getJobDataMap()).thenReturn(jobDataMap);
  }

  @Test
  public void shouldRemoveAttribute() {
    when(taskConfiguration.asMap()).thenReturn(ImmutableMap.of(FOO_KEY, FOO_VALUE, BAR_KEY, REMOVE_ATTRIBUTE_MARKER));
    when(jobDataMap.get(FOO_KEY)).thenReturn(FOO_VALUE);

    QuartzTaskUtils.updateJobData(jobDetail, taskConfiguration);

    verify(jobDataMap).remove(BAR_KEY);
    verify(jobDataMap, never()).put(anyString(), anyString());
  }

  @Test
  public void shouldUpdateAttribute() {
    String anotherFooValue = "anotherFooValue";

    when(taskConfiguration.asMap()).thenReturn(ImmutableMap.of(FOO_KEY, FOO_VALUE));
    when(jobDataMap.get(FOO_KEY)).thenReturn(anotherFooValue);

    QuartzTaskUtils.updateJobData(jobDetail, taskConfiguration);

    verify(jobDataMap).put(FOO_KEY, FOO_VALUE);
    verify(jobDataMap, never()).remove(BAR_KEY);
  }
}
