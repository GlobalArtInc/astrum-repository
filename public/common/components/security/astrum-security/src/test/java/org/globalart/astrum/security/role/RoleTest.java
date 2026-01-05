
package org.globalart.astrum.security.role;

import org.junit.Assert;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * Tests for {@link Role}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class RoleTest
{
  @Test
  public void testCompareDifferentId() throws Exception {
    Role roleA = new Role();
    roleA.setName("ID1");
    roleA.setRoleId("ID1");
    roleA.setSource("source");

    Role roleB = new Role();
    roleB.setName("ID2");
    roleB.setRoleId("ID2");
    roleB.setSource("source");

    Assert.assertEquals(-1, roleA.compareTo(roleB));
    Assert.assertEquals(1, roleB.compareTo(roleA));
  }

  @Test
  public void testCompareDifferentSource() throws Exception {
    Role roleA = new Role();
    roleA.setName("ID1");
    roleA.setRoleId("ID1");
    roleA.setSource("source1");

    Role roleB = new Role();
    roleB.setName("ID1");
    roleB.setRoleId("ID1");
    roleB.setSource("source2");

    Assert.assertEquals(-1, roleA.compareTo(roleB));
    Assert.assertEquals(1, roleB.compareTo(roleA));
  }
}
