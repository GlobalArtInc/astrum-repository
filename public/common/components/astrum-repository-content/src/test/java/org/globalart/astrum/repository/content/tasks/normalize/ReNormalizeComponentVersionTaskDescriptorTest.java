
package org.globalart.astrum.repository.content.tasks.normalize;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;

public class ReNormalizeComponentVersionTaskDescriptorTest
{
  @Test
  public void testTypeId() {
    assertThat(ReNormalizeComponentVersionTaskDescriptor.TYPE_ID, is(equalTo("component.renormalize.version")));
  }

  @Test
  public void testDescriptorProperties() {
    ReNormalizeComponentVersionTaskDescriptor underTest = new ReNormalizeComponentVersionTaskDescriptor();

    assertThat(underTest.getId(), is(equalTo(ReNormalizeComponentVersionTaskDescriptor.TYPE_ID)));
    assertThat(underTest.getType(), is(equalTo(ReNormalizeComponentVersionTask.class)));
    // internal repair task: never user-visible or exposed, unlike NormalizeComponentVersionTaskDescriptor which
    // supports opt-in visibility via astrum.component.normalize.{visible,expose}
    assertThat(underTest.isVisible(), is(false));
    assertThat(underTest.isExposed(), is(false));
  }
}
