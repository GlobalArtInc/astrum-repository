
package org.globalart.astrum.repository.view.payloads;

import java.io.IOException;
import java.io.InputStream;

import org.apache.http.HttpResponse;
import org.apache.http.message.BasicHeader;
import org.junit.Test;

import static com.google.common.net.HttpHeaders.CONTENT_LENGTH;
import static com.google.common.net.HttpHeaders.CONTENT_TYPE;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link HeaderOnlyPayload}
 */
public class HeaderOnlyPayloadTest
{
  @Test
  public void testHeadResponse_WithContentLengthAndType() throws IOException {
    // Simulate a typical HEAD response from a remote server
    HttpResponse response = mock(HttpResponse.class);
    when(response.getFirstHeader(CONTENT_LENGTH)).thenReturn(new BasicHeader(CONTENT_LENGTH, "1557794"));
    when(response.getFirstHeader(CONTENT_TYPE)).thenReturn(new BasicHeader(CONTENT_TYPE, "application/java-archive"));
    when(response.getEntity()).thenReturn(null);

    HeaderOnlyPayload payload = new HeaderOnlyPayload(response);

    // Verify size matches Content-Length header
    assertThat(payload.getSize(), is(1557794L));

    // Verify content type is preserved
    assertThat(payload.getContentType(), is("application/java-archive"));

    // Verify stream is empty (HEAD has no body)
    try (InputStream is = payload.openInputStream()) {
      assertThat(is.read(), is(-1));
    }

    // Verify close works
    payload.close();
  }

  @Test
  public void testHeadResponse_WithoutContentLength() {
    HttpResponse response = mock(HttpResponse.class);
    when(response.getFirstHeader(CONTENT_LENGTH)).thenReturn(null);
    when(response.getFirstHeader(CONTENT_TYPE)).thenReturn(new BasicHeader(CONTENT_TYPE, "text/plain"));

    HeaderOnlyPayload payload = new HeaderOnlyPayload(response);

    // When Content-Length is missing, size should be -1
    assertThat(payload.getSize(), is(-1L));
    assertThat(payload.getContentType(), is("text/plain"));
  }

  @Test
  public void testHeadResponse_WithInvalidContentLength() {
    HttpResponse response = mock(HttpResponse.class);
    when(response.getFirstHeader(CONTENT_LENGTH)).thenReturn(new BasicHeader(CONTENT_LENGTH, "not-a-number"));
    when(response.getFirstHeader(CONTENT_TYPE)).thenReturn(null);

    HeaderOnlyPayload payload = new HeaderOnlyPayload(response);

    // Invalid Content-Length should return -1
    assertThat(payload.getSize(), is(-1L));
    assertThat(payload.getContentType(), nullValue());
  }
}
