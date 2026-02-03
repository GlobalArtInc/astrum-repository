
package org.globalart.astrum.repository.view;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;

import javax.annotation.Nullable;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.MatcherAssert.assertThat;

public class PayloadTest
{
  private byte[] TEST_BYTES = "TEST CONTENT".getBytes();

  private ByteArrayInputStream input = new ByteArrayInputStream(TEST_BYTES);

  private ByteArrayOutputStream output;

  private Payload underTest;

  @Before
  public void setup() throws Exception {
    underTest = createTestPayload();
    output = new ByteArrayOutputStream();
  }

  @Test
  public void copy_Uses_Default_ByteCopy() throws IOException {
    underTest.copy(input, output);

    assertThat(output.toByteArray(), equalTo(TEST_BYTES));
  }

  private Payload createTestPayload() {
    return new Payload()
    {
      @Override
      public InputStream openInputStream() throws IOException {
        return null;
      }

      @Override
      public long getSize() {
        return 0;
      }

      @Nullable
      @Override
      public String getContentType() {
        return null;
      }
    };
  }
}
