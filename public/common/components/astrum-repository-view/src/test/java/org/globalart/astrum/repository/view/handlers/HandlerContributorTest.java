
package org.globalart.astrum.repository.view.handlers;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.StringPayload;

import org.junit.Test;
import org.mockito.ArgumentCaptor;

import static java.util.Arrays.asList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * UT for {@link HandlerContributor}.
 */
public class HandlerContributorTest
{
  @Test
  public void addContributedHandlers() throws Exception {
    final AttributesMap attributes = new AttributesMap();

    final Request request = mock(Request.class);
    final Context context = mock(Context.class);

    when(context.getRequest()).thenReturn(request);
    when(context.getAttributes()).thenReturn(attributes);
    when(context.proceed()).thenReturn(HttpResponses.ok(new Content(new StringPayload("test", "text/plain"))));

    final ContributedHandler handlerA = mock(ContributedHandler.class);
    final ContributedHandler handlerB = mock(ContributedHandler.class);
    final HandlerContributor underTest = new HandlerContributor(asList(handlerA, handlerB));
    final Response response = underTest.handle(context);

    assertThat(response.getStatus().isSuccessful(), is(true));
    assertThat(attributes.get(HandlerContributor.EXTENDED_MARKER), is(Boolean.TRUE));

    // Handle a second time to ensure the contributed handlers aren't injected twice
    underTest.handle(context);

    ArgumentCaptor<ContributedHandler> handlerCaptor = ArgumentCaptor.forClass(ContributedHandler.class);
    verify(context, times(2)).insertHandler(handlerCaptor.capture());
    assertThat(handlerCaptor.getAllValues(), is(asList(handlerB, handlerA))); // Intentionally added in "reverse" order
  }
}
