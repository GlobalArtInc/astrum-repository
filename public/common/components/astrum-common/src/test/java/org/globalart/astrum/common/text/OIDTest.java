
package org.globalart.astrum.common.text;

import java.util.Arrays;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.greaterThan;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertEquals;

/**
 * Tests for {@link OID}.
 */
public class OIDTest
{
  @Test
  public void testSimple() {
    Object obj = new Object();
    OID oid = OID.get(obj);
    assertEquals(obj.toString(), oid.toString());
  }

  @Test
  public void testParse() {
    Object obj = new Object();
    String spec = obj.toString();
    OID oid = OID.parse(spec);
    assertEquals(obj.getClass().getName(), oid.getType());
    assertEquals(obj.hashCode(), oid.getHash());
    assertEquals(spec, oid.toString());
  }

  @Test(expected = IllegalArgumentException.class)
  public void testParseIllegalOID() {
    Object obj = new Object();
    OID.parse(obj.toString() + "@illegal");
  }

  @Test
  public void testGetNull() {
    OID oid = OID.get(null);
    assertEquals(OID.NULL, oid);
  }

  @Test
  public void testRender() {
    Object obj = new Object();
    String repr = OID.render(obj);
    assertThat(repr, is(equalTo(obj.toString())));
    String prefix = Object.class.getName() + OID.SEPARATOR;
    assertThat(repr.startsWith(prefix), is(true));
    assertThat(repr.length(), greaterThan(prefix.length()));
  }

  @Test
  public void testFindById() {
    Object obj1 = new Object();
    Object obj2 = new Object();
    Object obj3 = new Object();

    assertThat(OID.find(Arrays.asList(obj1, obj2, obj3), OID.render(obj2)), is(equalTo(obj2)));
  }

  @Test
  public void testFindByOID() {
    Object obj1 = new Object();
    Object obj2 = new Object();
    Object obj3 = new Object();

    assertThat(OID.find(Arrays.asList(obj1, obj2, obj3), OID.oid(obj2)), is(equalTo(obj2)));
  }
}
