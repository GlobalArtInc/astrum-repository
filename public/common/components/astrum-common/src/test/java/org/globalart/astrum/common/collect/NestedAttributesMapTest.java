
package org.globalart.astrum.common.collect;

import java.util.HashMap;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.Assert.assertThrows;
import static org.globalart.astrum.common.collect.NestedAttributesMap.SEPARATOR;

/**
 * Tests for {@link org.globalart.astrum.common.collect.NestedAttributesMap}.
 */
public class NestedAttributesMapTest

{
  private NestedAttributesMap underTest;

  @Before
  public void setUp() {
    underTest = new NestedAttributesMap("foo", new HashMap<>());
  }

  /*
   * parentKey null when no parent"
   */
  @Test
  public void testParentKeyNullWhenNoParent() {
    assertThat(underTest.getKey(), is("foo"));
    assertThat(underTest.getParentKey(), nullValue());
  }

  /*
   * parentKey includes grandparent
   */
  @Test
  public void testParentKey_includesGrandparent() {
    NestedAttributesMap parent = underTest.child("bar");
    NestedAttributesMap child = parent.child("baz");

    assertThat(underTest.getKey(), is("foo"));
    assertThat(parent.getKey(), is("bar"));
    assertThat(child.getKey(), is("baz"));
    assertThat(child.getParentKey(), is("foo" + SEPARATOR + "bar"));
  }

  /*
   * qualifiedKey without parent returns key
   */
  @Test
  public void testQualifiedKey_withoutParent() {
    assertThat(underTest.getQualifiedKey(), is("foo"));
  }

  /*
   * qualifiedKey includes parent
   */
  @Test
  public void testQualifiedKey_withParent() {
    assertThat(underTest.child("bar").getQualifiedKey(), is("foo" + SEPARATOR + "bar"));
    assertThat(underTest.child("bar").child("baz").getQualifiedKey(),
        is("foo" + SEPARATOR + "bar" + SEPARATOR + "baz"));
  }

  @Test
  public void testChild_withNonMapField() {
    underTest.set("value", false);

    assertThrows(IllegalStateException.class, () -> underTest.child("value"));
  }

  /*
   * child require includes parent key
   */
  @Test
  public void testChildRequire_includesParentKey() {
    NestedAttributesMap bar = underTest.child("bar");
    Exception e = assertThrows(Exception.class, () -> bar.require("baz"));
    assertThat(e.getMessage(), containsString("baz"));
    assertThat(e.getMessage(), containsString("foo" + SEPARATOR + "bar"));

    NestedAttributesMap qux = underTest.child("bar").child("qux");
    e = assertThrows(Exception.class, () -> qux.require("baz"));
    assertThat(e.getMessage(), containsString("baz"));
    assertThat(e.getMessage(), containsString("foo" + SEPARATOR + "bar" + SEPARATOR + "qux"));
  }
}
