
package org.globalart.astrum.repository.content.tasks.normalize;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;

public class NormalizeComponentVersionTaskDescriptorTest
{
  @Test
  public void testTypeId() {
    assertThat(NormalizeComponentVersionTaskDescriptor.TYPE_ID, is(equalTo("component.normalize.version")));
  }

  @Test
  public void testDescriptorProperties() {
    NormalizeComponentVersionTaskDescriptor underTest =
        new NormalizeComponentVersionTaskDescriptor(false, false);

    assertThat(underTest.getId(), is(equalTo(NormalizeComponentVersionTaskDescriptor.TYPE_ID)));
    assertThat(underTest.getName(), is(equalTo("Repair - Normalize component versions for retain-n")));
    assertThat(underTest.getType(), is(equalTo(NormalizeComponentVersionTask.class)));
    assertThat(underTest.isVisible(), is(false));
    assertThat(underTest.isExposed(), is(false));
  }
}
