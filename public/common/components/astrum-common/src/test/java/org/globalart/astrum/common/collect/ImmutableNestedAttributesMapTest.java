
package org.globalart.astrum.common.collect;

import com.google.common.collect.Maps;
import org.junit.Test;

import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.MatcherAssert.assertThat;

/**
 * Test for the {@link ImmutableNestedAttributesMap}
 */
public class ImmutableNestedAttributesMapTest
{
  private ImmutableNestedAttributesMap map = new ImmutableNestedAttributesMap(null, "key", Maps.newHashMap());

  @Test(expected = UnsupportedOperationException.class)
  public void classKeysUnsettable() {
    map.set(Integer.class, 15);
  }

  @Test(expected = UnsupportedOperationException.class)
  public void stringKeysUnsettable() {
    map.set("key", "value");
  }

  @Test
  public void nonExistentChildrenAreNavigable() {
    final NestedAttributesMap nonexistent = map.child("nonexistent");
    assertThat(nonexistent, is(notNullValue()));
    assertThat(map.backing().isEmpty(), is(true));
  }

  @Test(expected = UnsupportedOperationException.class)
  public void navigableChildrenAreUnmodifiable() {
    map.child("nonexistent").set("key", "value");
  }
}
