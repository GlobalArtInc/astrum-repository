
package org.globalart.astrum.repository.routing.internal;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.routing.RoutingRuleHelper;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Parameters;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;

import com.google.common.collect.LinkedListMultimap;
import com.google.common.collect.ListMultimap;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertNotNull;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.nullable;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class RoutingRuleHandlerTest
{
  private static final String SOME_PATH = "/some/path";

  private RoutingRuleHandler underTest;

  @Mock
  private RoutingRuleHelper routingRuleHelper;

  @Mock
  private Context context;

  @Mock
  private Request request;

  @Mock
  private Response contextResponse;

  @Mock
  private Repository repository;

  @Before
  public void setup() throws Exception {
    underTest = new RoutingRuleHandler(routingRuleHelper);

    when(request.getPath()).thenReturn(SOME_PATH);
    when(context.getRequest()).thenReturn(request);
    when(request.getParameters()).thenReturn(new Parameters());
    when(context.proceed()).thenReturn(contextResponse);
    when(context.getRepository()).thenReturn(repository);
  }

  @Test
  public void testHandle_allowed() throws Exception {
    when(routingRuleHelper.isAllowed(nullable(Repository.class), eq(SOME_PATH))).thenReturn(true);

    Response response = underTest.handle(context);
    assertThat(response, is(contextResponse));
    verify(context).proceed();
  }

  @Test
  public void testHandle_blocked() throws Exception {
    when(routingRuleHelper.isAllowed(nullable(Repository.class), eq(SOME_PATH))).thenReturn(false);

    Type typeMock = mock(Type.class);
    when(repository.getType()).thenReturn(typeMock);
    when(typeMock.getValue()).thenReturn("repository-type");
    when(repository.getName()).thenReturn("repository-name");

    Response response = underTest.handle(context);

    assertNotNull(response);
    assertThat(response.getStatus().getCode(), is(403));
    verify(context, times(0)).proceed();
  }

  @Test
  public void testHandle_parameters() throws Exception {
    ListMultimap<String, String> params = LinkedListMultimap.create();
    params.put("foo", "bar");
    params.put("bar", "foo");
    when(request.getParameters()).thenReturn(new Parameters(params));
    when(routingRuleHelper.isAllowed(nullable(Repository.class), eq("/some/path?foo=bar&bar=foo"))).thenReturn(true);

    Response response = underTest.handle(context);
    assertThat(response, is(contextResponse));
    verify(context).proceed();
    verify(routingRuleHelper).isAllowed(nullable(Repository.class), eq("/some/path?foo=bar&bar=foo"));
  }
}
