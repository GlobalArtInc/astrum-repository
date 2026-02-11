
package org.globalart.astrum.repository.content.browse;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

public class TrimBrowseNodesTaskDescriptorTest
{
  @Test
  public void testTypeId() {
    assertThat(TrimBrowseNodesTaskDescriptor.TYPE_ID, is("repository.trim-browse-tree"));
  }

  @Test
  public void testDescriptorProperties() {
    TrimBrowseNodesTaskDescriptor underTest = new TrimBrowseNodesTaskDescriptor();
    assertThat(underTest.getId(), is("repository.trim-browse-tree"));
    assertThat(underTest.getName(), is(notNullValue()));
    assertThat(underTest.getType(), is(TrimBrowseNodesTask.class));
  }

  @Test
  public void testDoesNotAllowConcurrentRun() {
    TrimBrowseNodesTaskDescriptor underTest = new TrimBrowseNodesTaskDescriptor();
    assertThat(underTest.allowConcurrentRun(), is(false));
  }
}
