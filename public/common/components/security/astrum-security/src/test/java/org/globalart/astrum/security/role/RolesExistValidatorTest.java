
package org.globalart.astrum.security.role;

import java.util.Collections;

import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.authz.AuthorizationManager;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class RolesExistValidatorTest
{
  private RolesExistValidator underTest;

  @Before
  public void setup() throws Exception {
    SecuritySystem securitySystem = mock(SecuritySystem.class);
    AuthorizationManager authorizationManager = mock(AuthorizationManager.class);
    when(authorizationManager.getRole(any())).thenThrow(new NoSuchRoleException("test"));
    when(securitySystem.getAuthorizationManager(any())).thenReturn(authorizationManager);
    underTest = new RolesExistValidator(securitySystem);
  }

  @Test
  public void isValid_ignoresJavaElExpression() {
    ConstraintValidatorContext context = mock(ConstraintValidatorContext.class);
    when(context.buildConstraintViolationWithTemplate(any()))
        .thenReturn(mock(ConstraintValidatorContext.ConstraintViolationBuilder.class));
    assertThat(underTest.isValid(Collections
        .singleton("dx27e${\"gggggggggggggggggggggggggggggggggggggggggggz\".toString().replace(\"g\", \"q\")}yv5rm"),
        context), is(false));
    // note the missing $
    verify(context).buildConstraintViolationWithTemplate(
        "Missing roles: [dx27e{\"gggggggggggggggggggggggggggggggggggggggggggz\".toString().replace(\"g\", \"q\")}yv5rm]");
  }
}
