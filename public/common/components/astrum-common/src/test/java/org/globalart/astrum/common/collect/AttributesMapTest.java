
package org.globalart.astrum.common.collect;

import java.util.Date;

import org.junit.Before;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

/**
 * Tests for {@link org.globalart.astrum.common.collect.AttributesMap}.
 */
public class AttributesMapTest
{
  private AttributesMap underTest;

  @Before
  public void setUp() {
    underTest = new AttributesMap();
  }

  @Test
  public void testSetNullRemoves() {
    underTest.set("foo", "bar");
    underTest.set("foo", null);
    assertFalse(underTest.contains("foo"));
  }

  @Test
  public void testRequiredValue() {
    underTest.set("foo", "bar");
    assertEquals("bar", underTest.require("foo"));

    assertThrows(Exception.class, () -> {
      underTest.require("baz");
    });
  }

  @Test
  public void testContainsValue() {
    underTest.set("foo", "bar");
    assertTrue(underTest.contains("foo"));
    assertFalse(underTest.contains("baz"));
  }

  @Test
  public void testSizeEmptyAndClear() {
    assertTrue(underTest.isEmpty());
    assertEquals(0, underTest.size());

    underTest.set("foo", "bar");

    assertEquals(1, underTest.size());

    underTest.set("baz", "ick");

    assertEquals(2, underTest.size());

    underTest.clear();

    assertTrue(underTest.isEmpty());
    assertEquals(0, underTest.size());
  }

  @Test
  public void testTypedAccessors() {
    underTest.set("foo", 1);
    Object integerValue = underTest.get("foo", Integer.class);
    assertNotNull(integerValue);
    assertEquals(Integer.class, integerValue.getClass());
    assertEquals(1, integerValue);

    underTest.set("foo", false);
    Object booleanValue = underTest.get("foo", Boolean.class);
    assertNotNull(booleanValue);
    assertEquals(Boolean.class, booleanValue.getClass());
    assertEquals(false, booleanValue);
  }

  // private to test creation with accessible=true
  private static class GetOrCreateAttribute
  {
    // empty
  }

  @Test
  public void testGetOrCreate() {
    assertFalse(underTest.contains(GetOrCreateAttribute.class));
    Object value = underTest.getOrCreate(GetOrCreateAttribute.class);
    assertNotNull(value);
    assertTrue(underTest.contains(GetOrCreateAttribute.class));
  }

  @Test
  public void testHandleDateAsDateOrLong() {
    Date testDate = new Date();
    Long testDateLong = testDate.getTime();

    underTest.set("foo", testDate);
    underTest.set("bar", testDateLong);

    assertEquals(testDate, underTest.get("foo", Date.class));
    assertEquals(testDate, underTest.get("bar", Date.class));
  }

  @Test
  public void testHandleBooleanStrings() {
    underTest.set("test", "true");
    assertEquals(true, underTest.get("test", Boolean.class));

    underTest.set("test", "false");
    assertEquals(false, underTest.get("test", Boolean.class));

    underTest.set("test", "notABooleanString");
    assertEquals(false, underTest.get("test", Boolean.class));

    underTest.set("test", 123);
    assertNull(underTest.get("test", Boolean.class));
  }

  private Object testComputeFunction(Object test) {
    if (test instanceof Integer) {
      return (Integer) test + 1;
    }
    return 1;
  }

  @Test
  public void testCompute() {
    Object result = underTest.compute("foo", this::testComputeFunction);
    assertNull(result);
    assertEquals(1, underTest.get("foo"));
    result = underTest.compute("foo", this::testComputeFunction);
    assertEquals(1, result);
    assertEquals(2, underTest.get("foo"));
    result = underTest.compute("foo", this::testComputeFunction);
    assertEquals(2, result);
    assertEquals(3, underTest.get("foo"));
    result = underTest.compute("foo", this::testComputeFunction);
    assertEquals(3, result);
    assertEquals(4, underTest.get("foo"));
  }
}
