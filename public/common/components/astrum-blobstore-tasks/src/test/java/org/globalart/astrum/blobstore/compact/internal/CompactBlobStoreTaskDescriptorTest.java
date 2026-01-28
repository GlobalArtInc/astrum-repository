
package org.globalart.astrum.blobstore.compact.internal;

import org.globalart.astrum.scheduling.TaskConfiguration;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.scheduling.TaskDescriptorSupport.MULTINODE_KEY;

class CompactBlobStoreTaskDescriptorTest

{
  private CompactBlobStoreTaskDescriptor underTest;

  private TaskConfiguration taskConfiguration = new TaskConfiguration();

  @Test
  void initializeConfiguration() {
    underTest = new CompactBlobStoreTaskDescriptor(true);
    underTest.initializeConfiguration(taskConfiguration);
    assertThat(taskConfiguration.getBoolean(MULTINODE_KEY, false), is(false));
  }

  @Test
  void testVisibleAndExposedWhenEnabled() {
    underTest = new CompactBlobStoreTaskDescriptor(true);

    assertThat("Task should be visible when enabled", underTest.isVisible(), is(true));
    assertThat("Task should be exposed when enabled", underTest.isExposed(), is(true));
  }

  @Test
  void testNotVisibleAndNotExposedWhenDisabled() {
    underTest = new CompactBlobStoreTaskDescriptor(false);

    assertThat("Task should not be visible when disabled", underTest.isVisible(), is(false));
    assertThat("Task should not be exposed when disabled", underTest.isExposed(), is(false));
  }
}
