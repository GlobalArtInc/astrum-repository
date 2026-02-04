
package org.globalart.astrum.repository.view.handlers;

import org.globalart.astrum.repository.date.DateTimeUtils;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.StringPayload;

import com.google.common.net.HttpHeaders;
import org.joda.time.DateTime;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * UT for {@link ContentHeadersHandler}.
 *
 * @since 3.0
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class ContentHeadersHandlerTest
{
  final DateTime now = DateTime.now();

  final ContentHeadersHandler subject = new ContentHeadersHandler();

  final String payloadString = "testPayload";

  @Mock
  Context context;

  @Mock
  Request request;

  @Before
  public void before() throws Exception {
    when(context.getRequest()).thenReturn(request);
  }

  @Test
  public void okResponse() throws Exception {
    final Content content = new Content(new StringPayload(payloadString, "text/plain"));
    content.getAttributes().set(Content.CONTENT_LAST_MODIFIED, now);
    content.getAttributes().set(Content.CONTENT_ETAG, "etag");
    when(context.proceed()).thenReturn(HttpResponses.ok(content));
    final Response r = subject.handle(context);
    assertThat(r.getStatus().isSuccessful(), is(true));
    assertThat(r.getHeaders().get(HttpHeaders.LAST_MODIFIED), equalTo(DateTimeUtils.formatDateTime(now)));
    assertThat(r.getHeaders().get(HttpHeaders.ETAG), equalTo("\"etag\""));
  }

  @Test
  public void okResponseWithWeakEtag() throws Exception {
    final Content content = new Content(new StringPayload(payloadString, "text/plain"));
    content.getAttributes().set(Content.CONTENT_LAST_MODIFIED, now);
    content.getAttributes().set(Content.CONTENT_ETAG, "W/\"etag\"");
    when(context.proceed()).thenReturn(HttpResponses.ok(content));
    final Response r = subject.handle(context);
    assertThat(r.getStatus().isSuccessful(), is(true));
    assertThat(r.getHeaders().get(HttpHeaders.LAST_MODIFIED), equalTo(DateTimeUtils.formatDateTime(now)));
    assertThat(r.getHeaders().get(HttpHeaders.ETAG), equalTo("W/\"etag\""));
  }

  @Test
  public void okResponseNoExtraData() throws Exception {
    when(context.proceed()).thenReturn(
        HttpResponses.ok(new Content(new StringPayload(payloadString, "text/plain"))));
    final Response r = subject.handle(context);
    assertThat(r.getStatus().isSuccessful(), is(true));
    assertThat(r.getHeaders().get(HttpHeaders.LAST_MODIFIED), nullValue());
    assertThat(r.getHeaders().get(HttpHeaders.ETAG), nullValue());
  }

  @Test
  public void okResponseNotContent() throws Exception {
    when(context.proceed()).thenReturn(HttpResponses.ok(new StringPayload("test", "text/plain")));
    final Response r = subject.handle(context);
    assertThat(r.getStatus().isSuccessful(), is(true));
    assertThat(r.getHeaders().get(HttpHeaders.LAST_MODIFIED), nullValue());
    assertThat(r.getHeaders().get(HttpHeaders.ETAG), nullValue());
  }

  @Test
  public void notOkResponse() throws Exception {
    when(context.proceed()).thenReturn(HttpResponses.notFound());
    final Response r = subject.handle(context);
    assertThat(r.getStatus().isSuccessful(), is(false));
    assertThat(r.getHeaders().get(HttpHeaders.LAST_MODIFIED), nullValue());
    assertThat(r.getHeaders().get(HttpHeaders.ETAG), nullValue());
  }
}
