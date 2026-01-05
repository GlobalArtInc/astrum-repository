
package org.globalart.astrum.security.privilege;

import java.util.Collections;
import java.util.HashSet;
import java.util.Set;

import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.security.SecuritySystem;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class PrivilegesExistValidatorTest
{
  private PrivilegesExistValidator underTest;

  private SecuritySystem securitySystem;

  @Before
  public void setup() {
    securitySystem = mock(SecuritySystem.class);
    when(securitySystem.listPrivileges()).thenReturn(Collections.emptySet());
    underTest = new PrivilegesExistValidator(securitySystem);
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
        "Invalid privilege id: dx27e{\"gggggggggggggggggggggggggggggggggggggggggggz\".toString().replace(\"g\", \"q\")}yv5rm. Only letters, digits, underscores(_), hyphens(-), dots(.), and asterisks(*) are allowed and may not start with underscore or dot.");
  }

  @Test
  public void isValid_allows_wildcards() {
    Set<Privilege> validPrivileges = new HashSet<>();
    Privilege privilege = mock(Privilege.class);
    when(privilege.getId()).thenReturn("as-repository-admin-maven2-maven-public-*");
    validPrivileges.add(privilege);
    when(securitySystem.listPrivileges()).thenReturn(validPrivileges);
    ConstraintValidatorContext context = mock(ConstraintValidatorContext.class);
    when(context.buildConstraintViolationWithTemplate(any()))
        .thenReturn(mock(ConstraintValidatorContext.ConstraintViolationBuilder.class));
    assertThat(underTest.isValid(Collections
        .singleton("as-repository-admin-maven2-maven-public-*"),
        context), is(true));
  }
}
