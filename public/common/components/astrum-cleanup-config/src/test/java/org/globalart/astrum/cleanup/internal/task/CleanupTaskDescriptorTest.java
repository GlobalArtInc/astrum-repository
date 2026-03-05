
package org.globalart.astrum.cleanup.internal.task;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.cleanup.internal.task.CleanupTaskDescriptor.TYPE_ID;

public class CleanupTaskDescriptorTest
{
  private CleanupTaskDescriptor underTest;

  @Before
  public void setup() {
    underTest = new CleanupTaskDescriptor();
  }

  @Test
  public void typeIdConstantHasExpectedValue() {
    assertThat(TYPE_ID, is("repository.cleanup"));
  }

  @Test
  public void descriptorReportsExpectedId() {
    assertThat(underTest.getId(), is(TYPE_ID));
    assertThat(underTest.getId(), is("repository.cleanup"));
  }

  @Test
  public void descriptorReportsExpectedName() {
    assertThat(underTest.getName(), is("Admin - Cleanup repositories using their associated policies"));
  }

  @Test
  public void descriptorReportsExpectedType() {
    assertThat(underTest.getType(), is(CleanupTask.class));
  }

  @Test
  public void descriptorIsVisible() {
    assertThat(underTest.isVisible(), is(true));
  }

  @Test
  public void descriptorIsNotExposed() {
    assertThat(underTest.isExposed(), is(false));
  }

  @Test
  public void descriptorHasNoFormFields() {
    assertThat(underTest.getFormFields(), is(empty()));
  }
}
