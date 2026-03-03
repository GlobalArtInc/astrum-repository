
package org.globalart.astrum.repository.security;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.view.Request;

import org.apache.shiro.authz.AuthorizationException;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.junit.Assert.fail;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.security.BreadActions.READ;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SecurityFacetSupportTest
{
  private static final class TestSecurityFacetSupport
      extends SecurityFacetSupport
  {
    public TestSecurityFacetSupport(
        final RepositoryFormatSecurityContributor securityContributor,
        final VariableResolverAdapter variableResolverAdapter,
        final ContentPermissionChecker contentPermissionChecker)
    {
      super(securityContributor, variableResolverAdapter, contentPermissionChecker);
    }
  }

  @Mock
  Request request;

  @Mock
  Repository repository;

  @Mock
  ContentPermissionChecker contentPermissionChecker;

  @Mock
  VariableResolverAdapter variableResolverAdapter;

  @Mock
  RepositoryFormatSecurityContributor securityContributor;

  TestSecurityFacetSupport testSecurityFacetSupport;

  @Before
  public void setupConfig() throws Exception {
    when(request.getPath()).thenReturn("/some/path.txt");
    when(request.getAction()).thenReturn(HttpMethods.GET);

    when(repository.getFormat()).thenReturn(new Format("test")
    {
    });
    when(repository.getName()).thenReturn("SecurityFacetSupportTest");

    testSecurityFacetSupport = new TestSecurityFacetSupport(securityContributor,
        variableResolverAdapter, contentPermissionChecker);

    testSecurityFacetSupport.attach(repository);
  }

  @Test
  public void testEnsurePermitted_permitted() throws Exception {
    when(contentPermissionChecker.isPermitted(eq("SecurityFacetSupportTest"), eq("test"), eq(READ), any(), any()))
        .thenReturn(true);
    try {
      testSecurityFacetSupport.ensurePermitted(request);
    }
    catch (Exception e) {
      fail("permitted action should have been permitted");
    }
  }

  @Test
  public void testEnsurePermitted_notPermitted() throws Exception {
    when(contentPermissionChecker.isPermitted(eq("SecurityFacetSupportTest"), eq("test"), eq(READ), any(), any()))
        .thenReturn(false);

    try {
      testSecurityFacetSupport.ensurePermitted(request);
      fail("AuthorizationException should have been thrown");
    }
    catch (AuthorizationException e) {
      // expected
    }

    verify(contentPermissionChecker).isPermitted(eq("SecurityFacetSupportTest"), eq("test"), eq(READ), any(), any());
  }
}
