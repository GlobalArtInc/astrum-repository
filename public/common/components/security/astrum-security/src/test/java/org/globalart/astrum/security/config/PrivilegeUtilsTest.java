
package org.globalart.astrum.security.config;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

public class PrivilegeUtilsTest
{
  @Test
  public void testCleanName() {
    assertThat(PrivilegeUtils.cleanName(" some name!@#$%^&*() "), is("some_name"));
  }

  @Test
  public void testCleanName_null() {
    // prior to ASTRUM-10733 this would throw an NPE
    assertThat(PrivilegeUtils.cleanName(null), nullValue());
  }
}
