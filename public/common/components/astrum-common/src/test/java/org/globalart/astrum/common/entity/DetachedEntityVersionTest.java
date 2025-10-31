
package org.globalart.astrum.common.entity;

import org.junit.Test;

import static org.junit.Assert.*;

/**
 * Tests for {@link DetachedEntityVersion}
 */
public class DetachedEntityVersionTest

{
  @Test
  public void testDetachedEquality() {
    DetachedEntityVersion a = new DetachedEntityVersion("a");
    assertTrue(a.equals(a));
    assertTrue(a.equals(new DetachedEntityVersion("a")));

    DetachedEntityVersion b = new DetachedEntityVersion("b");
    assertTrue(!a.equals(b));
  }
}
