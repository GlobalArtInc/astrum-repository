
package org.globalart.astrum.repository;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;

public class ETagHeaderUtilsTest
{
  @Test
  public void quoteStrong() {
    assertEquals("\"foobar\"", ETagHeaderUtils.quote("foobar"));
  }

  @Test
  public void quoteWeak() {
    assertEquals("W/\"foobar\"", ETagHeaderUtils.quote("W/\"foobar\""));
  }

  @Test
  public void extractNull() {
    assertNull(ETagHeaderUtils.extract(null));
  }

  @Test
  public void extractEmpty() {
    assertEquals("", ETagHeaderUtils.extract(""));
  }

  @Test
  public void extractStrong() {
    assertEquals("foobar", ETagHeaderUtils.extract("\"foobar\""));
  }

  @Test
  public void extractWeak() {
    assertEquals("W/\"foobar\"", ETagHeaderUtils.extract("W/\"foobar\""));
  }
}
