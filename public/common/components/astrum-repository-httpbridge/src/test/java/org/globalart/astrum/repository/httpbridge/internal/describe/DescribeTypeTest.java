
package org.globalart.astrum.repository.httpbridge.internal.describe;

import org.junit.Test;

import static org.junit.Assert.assertEquals;

/**
 * Tests for {@link DescribeType}.
 */
public class DescribeTypeTest
{

  @Test
  public void parseHtml() {
    assertEquals(DescribeType.HTML, DescribeType.parse("html"));
  }

  @Test
  public void parseJson() {
    assertEquals(DescribeType.JSON, DescribeType.parse("json"));
  }

  @Test
  public void parseBlankAsHtml() {
    assertEquals(DescribeType.HTML, DescribeType.parse(""));
  }

  @Test
  public void parseTrueAsHtml() {
    assertEquals(DescribeType.HTML, DescribeType.parse("true"));
  }
}
