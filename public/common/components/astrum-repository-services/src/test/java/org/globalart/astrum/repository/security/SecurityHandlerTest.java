
package org.globalart.astrum.repository.security;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.atMostOnce;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SecurityHandlerTest
{
  private SecurityHandler underTest;

  @Mock
  private Context context;

  @Mock
  private Repository repository;

  @Mock
  private SecurityFacet securityFacet;

  @Mock
  private Handler loginsCounterHandler;

  private AttributesMap attributesMap;

  @Before
  public void setup() {
    underTest = new SecurityHandler(null);

    attributesMap = new AttributesMap();
    when(repository.facet(SecurityFacet.class)).thenReturn(securityFacet);
    when(context.getRepository()).thenReturn(repository);
    when(context.getAttributes()).thenReturn(attributesMap);
  }

  @Test
  public void testHandle() throws Exception {
    underTest.handle(context);
    verify(securityFacet).ensurePermitted(any(), any(SelectorEvaluationCache.class));
  }

  @Test
  public void testHandle_alreadyAuthorized() throws Exception {
    attributesMap.set(SecurityHandler.AUTHORIZED_KEY, true);
    underTest.handle(context);
    verify(securityFacet, never()).ensurePermitted(any(), any());
  }

  @Test
  public void testHandle_loginsCounterHandlerIsNull() throws Exception {
    underTest.handle(context);
    verify(context, never()).insertHandler(loginsCounterHandler);
  }

  @Test
  public void testHandle_loginsCounterHandlerNonNull() throws Exception {
    underTest = new SecurityHandler(loginsCounterHandler);
    underTest.handle(context);
    verify(context, atMostOnce()).insertHandler(loginsCounterHandler);
  }
}
