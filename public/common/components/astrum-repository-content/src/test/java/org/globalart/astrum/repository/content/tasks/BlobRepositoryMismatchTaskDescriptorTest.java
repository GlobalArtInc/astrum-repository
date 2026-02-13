
package org.globalart.astrum.repository.content.tasks;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

public class BlobRepositoryMismatchTaskDescriptorTest
{
  @Test
  public void testTypeIdConstant() {
    assertThat(BlobRepositoryMismatchTaskDescriptor.TYPE_ID, is("repository.blob.mismatch.task"));
  }

  @Test
  public void testConstructorExposed() {
    BlobRepositoryMismatchTaskDescriptor underTest = new BlobRepositoryMismatchTaskDescriptor(true);
    assertThat(underTest, is(notNullValue()));
    assertThat(underTest.getId(), is("repository.blob.mismatch.task"));
    assertThat(underTest.getName(), containsString("blob"));
  }

  @Test
  public void testConstructorNotExposed() {
    BlobRepositoryMismatchTaskDescriptor underTest = new BlobRepositoryMismatchTaskDescriptor(false);
    assertThat(underTest, is(notNullValue()));
    assertThat(underTest.getId(), is("repository.blob.mismatch.task"));
  }

  @Test
  public void testFormFields() {
    BlobRepositoryMismatchTaskDescriptor underTest = new BlobRepositoryMismatchTaskDescriptor(true);
    assertThat(underTest.getFormFields(), is(notNullValue()));
  }
}
