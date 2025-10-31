
package org.globalart.astrum.common.entity;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;

/**
 * Tests for {@link DetachedEntityId}
 */
public class DetachedEntityIdTest

{
  @Test
  public void testDetachedEquality() {
    DetachedEntityId a = new DetachedEntityId("a");
    assertThat(a, is(a));
    assertThat(a, is(new DetachedEntityId("a")));

    DetachedEntityId b = new DetachedEntityId("b");
    assertThat(a, not(b));
  }
}
