
package org.globalart.astrum.blobstore.api;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.arrayContaining;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.sameInstance;

/**
 * Tests for {@link OperationType}.
 */
public class OperationTypeTest
{
  @Test
  public void testValues() {
    assertThat(OperationType.values(), arrayContaining(OperationType.UPLOAD, OperationType.DOWNLOAD));
  }

  @Test
  public void testValueOf() {
    assertThat(OperationType.valueOf("UPLOAD"), is(sameInstance(OperationType.UPLOAD)));
    assertThat(OperationType.valueOf("DOWNLOAD"), is(sameInstance(OperationType.DOWNLOAD)));
  }

  @Test(expected = IllegalArgumentException.class)
  public void testValueOfUnknownThrows() {
    OperationType.valueOf("UNKNOWN");
  }

  @Test(expected = NullPointerException.class)
  public void testValueOfNullThrows() {
    OperationType.valueOf(null);
  }

  @Test
  public void testNameAndOrdinal() {
    assertThat(OperationType.UPLOAD.name(), is("UPLOAD"));
    assertThat(OperationType.UPLOAD.ordinal(), is(0));
    assertThat(OperationType.DOWNLOAD.name(), is("DOWNLOAD"));
    assertThat(OperationType.DOWNLOAD.ordinal(), is(1));
  }
}
