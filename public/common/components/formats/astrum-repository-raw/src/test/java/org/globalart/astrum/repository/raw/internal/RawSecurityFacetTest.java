
package org.globalart.astrum.repository.raw.internal;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.view.Request;

import org.apache.shiro.authz.AuthorizationException;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.fail;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.security.BreadActions.READ;

@RunWith(MockitoJUnitRunner.Silent.class)
public class RawSecurityFacetTest

{
  @Mock
  Request request;

  @Mock
  Repository repository;

  @Mock
  ContentPermissionChecker contentPermissionChecker;

  @Mock
  VariableResolverAdapter variableResolverAdapter;

  @Mock
  RawFormatSecurityContributor securityContributor;

  RawSecurityFacet rawSecurityFacet;

  @Before
  public void setupConfig() throws Exception {
    when(request.getPath()).thenReturn("/some/path.txt");
    when(request.getAction()).thenReturn(HttpMethods.GET);

    when(repository.getFormat()).thenReturn(new Format("raw")
    {
    });
    when(repository.getName()).thenReturn("RawSecurityFacetTest");

    rawSecurityFacet = new RawSecurityFacet(securityContributor,
        variableResolverAdapter, contentPermissionChecker);

    rawSecurityFacet.attach(repository);
  }

  @Test
  public void testEnsurePermitted_permitted() throws Exception {
    when(contentPermissionChecker.isPermitted(eq("RawSecurityFacetTest"), eq("raw"), eq(READ), any(), any()))
        .thenReturn(true);

    try {
      rawSecurityFacet.ensurePermitted(request);
    }
    catch (AuthorizationException e) {
      fail("expected permitted operation to succeed");
    }
  }

  @Test
  public void testEnsurePermitted_notPermitted() throws Exception {
    when(contentPermissionChecker.isPermitted(eq("RawSecurityFacetTest"), eq("raw"), eq(READ), any(), any()))
        .thenReturn(false);
    try {
      rawSecurityFacet.ensurePermitted(request);
      fail("AuthorizationException should have been thrown");
    }
    catch (AuthorizationException e) {
      // expected
    }

    verify(contentPermissionChecker).isPermitted(eq("RawSecurityFacetTest"), eq("raw"), eq(READ), any(), any());
  }
}
