
package org.globalart.astrum.repository.raw;

import java.util.Arrays;
import java.util.Collection;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;

import static org.junit.Assert.assertEquals;

/**
 * {@link RawCoordinatesHelper} tests.
 */
@RunWith(Parameterized.class)
public class RawCoordinatesHelperTest
{

  @Parameterized.Parameter(0)
  public String path;

  @Parameterized.Parameter(1)
  public String expectedGroup;

  @Parameterized.Parameters(name = "group of {0} is {1}")
  public static Collection<Object[]> data() {
    return Arrays.asList(new Object[][]{
        {"/foo/bar", "/foo"},
        {"foo/bar", "/foo"},
        {"foobar.txt", "/"},
        {"/foobar.txt", "/"},
        {"/some/long/involved/path.txt", "/some/long/involved"},
        {"some/long/involved/path.txt", "/some/long/involved"}
    });
  }

  @Test
  public void testGetGroup() {
    String group = RawCoordinatesHelper.getGroup(path);
    assertEquals(expectedGroup, group);
  }
}
