
package org.globalart.astrum.repository.content.tasks;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

public class CreateAssetBlobIndexTaskDescriptorTest
{
  private CreateAssetBlobIndexTaskDescriptor underTest;

  @Test
  public void testNotExposedByDefault() {
    underTest = new CreateAssetBlobIndexTaskDescriptor(false);

    assertThat("Task should not be exposed by default", underTest.isExposed(), is(false));
    assertThat("Task should be visible", underTest.isVisible(), is(true));
  }

  @Test
  public void testVisibleAndExposedWhenEnabled() {
    underTest = new CreateAssetBlobIndexTaskDescriptor(true);

    assertThat("Task should be visible when enabled", underTest.isVisible(), is(true));
    assertThat("Task should be exposed when enabled", underTest.isExposed(), is(true));
  }

  @Test
  public void testTypeId() {
    underTest = new CreateAssetBlobIndexTaskDescriptor(false);

    assertThat("Type ID should be set correctly",
        underTest.getId(),
        equalTo(CreateAssetBlobIndexTaskDescriptor.TYPE_ID));
    assertThat("Type ID should be repository.asset.blob.index.migration",
        underTest.getId(),
        equalTo("repository.asset.blob.index.migration"));
  }

  @Test
  public void testTaskType() {
    underTest = new CreateAssetBlobIndexTaskDescriptor(false);

    assertThat("Task type should be CreateAssetBlobIndexTask",
        underTest.getType(),
        equalTo(CreateAssetBlobIndexTask.class));
  }

  @Test
  public void testName() {
    underTest = new CreateAssetBlobIndexTaskDescriptor(false);

    assertThat("Task name should be set", underTest.getName(), notNullValue());
    assertThat("Task name should describe the task",
        underTest.getName(),
        equalTo("Create asset blob index for blob_created and asset_blob_id"));
  }
}
