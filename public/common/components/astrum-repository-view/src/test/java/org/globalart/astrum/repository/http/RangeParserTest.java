
package org.globalart.astrum.repository.http;

import java.util.List;

import com.google.common.collect.Range;
import org.junit.Test;

import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;

/**
 * Tests {@link RangeParser}.
 */
public class RangeParserTest
{
  private RangeParser parser = new RangeParser();

  @Test
  public void first500Bytes() {
    final List<Range<Long>> ranges = parser.parseRangeSpec("bytes=0-499", 1000L);
    assertThat(ranges.size(), is(1));
    assertThat(ranges.get(0), is(Range.closed(0L, 499L)));
  }

  @Test
  public void second500Bytes() {
    final List<Range<Long>> ranges = parser.parseRangeSpec("bytes=500-999", 1000L);
    assertThat(ranges.size(), is(1));
    assertThat(ranges.get(0), is(Range.closed(500L, 999L)));
  }

  @Test
  public void last500Bytes() {
    final List<Range<Long>> ranges = parser.parseRangeSpec("bytes=9500-", 10000L);
    assertThat(ranges.size(), is(1));
    assertThat(ranges.get(0), is(Range.closed(9500L, 9999L)));
  }

  @Test
  public void last500BytesSuffix() {
    final List<Range<Long>> ranges = parser.parseRangeSpec("bytes=-1", 10L);
    assertThat(ranges.size(), is(1));
    assertThat(ranges.get(0), is(Range.closed(9L, 9L)));
  }

  @Test
  public void completeSuffix() {
    final Range<Long> fullContent = parser.parseRangeSpec("bytes=0-", 10L).get(0);
    final Range<Long> fullSuffix = parser.parseRangeSpec("bytes=-10", 10L).get(0);
    assertThat(fullSuffix, is(fullContent));
  }

  @Test
  public void illegalRange() {
    final List<Range<Long>> ranges = parser.parseRangeSpec("bytes=10-", 5L);
    System.err.println(ranges);
    assertThat(ranges, is(nullValue()));
  }

  @Test
  public void suffixTooLarge() {
    final List<Range<Long>> ranges = parser.parseRangeSpec("bytes=-10", 5L);
    System.err.println(ranges);
    assertThat(ranges, is(nullValue()));
  }

  // The RFC allows for ranges that exceed the end of the content to be fulfilled
  @Test
  public void partialOverlapsAreOkay() {
    final List<Range<Long>> ranges = parser.parseRangeSpec("bytes=5-100", 10L);
    assertThat(ranges.get(0), is(Range.closed(5L, 9L)));
  }
}
