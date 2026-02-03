
package org.globalart.astrum.repository.http;

import java.util.Arrays;
import java.util.Collection;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Headers;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.view.handlers.ConditionalRequestHandler;

import com.google.common.collect.LinkedListMultimap;
import com.google.common.collect.ListMultimap;
import com.google.common.net.HttpHeaders;
import org.apache.http.client.utils.DateUtils;
import org.joda.time.DateTime;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;
import org.junit.runners.Parameterized.Parameter;
import org.junit.runners.Parameterized.Parameters;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnit;
import org.mockito.junit.MockitoRule;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@RunWith(Parameterized.class)
public class ConditionalRequestHandlerTest
{
  private static final String WILDCARD = "*";

  private static final String MATCH_ETAG = "foo, bar";

  private static final String MATCH_DIFF = "bar";

  private static final String ETAG = "foo";

  private static final String OLDER_TIME = DateUtils.formatDate(DateTime.now().toDate());

  private static final String NEWER_TIME = DateUtils.formatDate(DateTime.now().plusHours(1).toDate());

  // PUT/POST requests first do a GET request before deciding whether to proceed
  private static final int FAIL_STATUS = 999;

  @Parameters(name = "{index}: {8}")
  public static Collection<Object[]> data() {
    return Arrays.asList(new Object[][]{
        {null, null, null, null, null, null, 200, 200, "No relevant headers set"},

        /* Proper null handling on request headers */
        {NEWER_TIME, null, null, null, null, null, 200, 200, "Handle null request modified headers"},
        {null, null, null, ETAG, null, null, 200, 200, "Handle null request match headers"},

        {NEWER_TIME, OLDER_TIME, null, null, null, null, 200, 200, "Last-Modified newer than If-Modified-Since"},
        {NEWER_TIME, null, OLDER_TIME, null, null, null, 412, 412, "Last-Modified newer than If-Unmodified-Since"},

        {OLDER_TIME, OLDER_TIME, null, null, null, null, 304, 200, "Last-Modified same as If-Modified-Since"},
        {OLDER_TIME, null, OLDER_TIME, null, null, null, 200, 200, "Last-Modified same as If-Unmodified-Since"},

        {null, null, null, ETAG, MATCH_ETAG, null, 200, 200, "E-Tag same as If-Match"},
        {null, null, null, ETAG, null, MATCH_ETAG, 304, 412, "E-Tag same as If-None-Match"},

        {null, null, null, ETAG, MATCH_DIFF, null, 412, 412, "E-Tag different from If-Match"},
        {null, null, null, ETAG, null, MATCH_DIFF, 200, 200, "E-Tag different from If-None-Match"},

        {null, null, null, ETAG, WILDCARD, null, 200, 200, "E-Tag with wildcard If-Match"},
        {null, null, null, ETAG, null, WILDCARD, 304, 412, "E-Tag with wildcard If-None-Match"}
    });
  }

  @Parameter(0)
  public String responseModified;

  @Parameter(1)
  public String ifModifiedSince;

  @Parameter(2)
  public String ifUnmodifiedSince;

  @Parameter(3)
  public String responseEtag;

  @Parameter(4)
  public String ifMatch;

  @Parameter(5)
  public String ifNonMatch;

  @Parameter(6)
  public int headGetStatus;

  @Parameter(7)
  public int putPostStatus;

  @Parameter(8)
  public String testName;

  @Rule
  public MockitoRule mockitoRule = MockitoJUnit.rule().silent();

  @Mock
  private Context context;

  @Mock
  private Request request;

  @Mock
  private Response contextProceedResponse;

  @Mock
  private Response dispatchedResponse;

  @Mock
  private Headers respHeaders;

  private ConditionalRequestHandler underTest;

  @Before
  public void setup() throws Exception {
    underTest = new ConditionalRequestHandler();

    when(context.proceed()).thenReturn(contextProceedResponse);
    when(context.getRequest()).thenReturn(request);

    ListMultimap<String, String> entries = LinkedListMultimap.create();
    Headers reqHeaders = new Headers(entries);
    when(request.getHeaders()).thenReturn(reqHeaders);
    when(request.getAttributes()).thenReturn(new AttributesMap());
    when(request.getPath()).thenReturn("/org/apache/httpcomponents/httpcomponents.jar");

    Repository repository = mock(Repository.class);
    when(context.getRepository()).thenReturn(repository);

    ViewFacet viewFacet = mock(ViewFacet.class);
    when(repository.facet(ViewFacet.class)).thenReturn(viewFacet);
    when(viewFacet.dispatch(any())).thenReturn(dispatchedResponse);

    if (ifModifiedSince != null) {
      entries.put(HttpHeaders.IF_MODIFIED_SINCE, ifModifiedSince);
    }
    if (ifUnmodifiedSince != null) {
      entries.put(HttpHeaders.IF_UNMODIFIED_SINCE, ifUnmodifiedSince);
    }
    if (ifNonMatch != null) {
      entries.put(HttpHeaders.IF_NONE_MATCH, ifNonMatch);
    }
    if (ifMatch != null) {
      entries.put(HttpHeaders.IF_MATCH, ifMatch);
    }

    when(contextProceedResponse.getHeaders()).thenReturn(respHeaders);
    when(dispatchedResponse.getHeaders()).thenReturn(respHeaders);
    when(respHeaders.get(HttpHeaders.ETAG)).thenReturn(responseEtag);
    when(respHeaders.get(HttpHeaders.LAST_MODIFIED)).thenReturn(responseModified);
  }

  @Test
  public void testShouldReturnContent_Get() throws Exception {
    assertAction("GET", headGetStatus, 200, FAIL_STATUS);
  }

  @Test
  public void testShouldReturnContent_Head() throws Exception {
    assertAction("HEAD", headGetStatus, 200, FAIL_STATUS);
  }

  @Test
  public void testShouldReturnContent_Post() throws Exception {
    assertAction("POST", putPostStatus, 200, 200);
  }

  @Test
  public void testShouldReturnContent_Put() throws Exception {
    assertAction("PUT", putPostStatus, 200, 200);
  }

  private void assertAction(
      final String action,
      final int expectedStatus,
      final int proceedStatus,
      final int dispatchedStatus) throws Exception
  {
    when(contextProceedResponse.getStatus()).thenReturn(status(proceedStatus));
    when(dispatchedResponse.getStatus()).thenReturn(status(dispatchedStatus));

    when(request.getAction()).thenReturn(action);
    Response response = underTest.handle(context);
    assertThat(response.getStatus().getCode(), is(expectedStatus));
  }

  private static Status status(final int code) {
    if (code < 400) {
      return Status.success(code);
    }
    return Status.failure(code);
  }
}
