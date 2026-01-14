
package org.globalart.astrum.audit.internal;

import java.util.Date;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.globalart.astrum.audit.AuditData;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.joda.time.DateTime.parse;
import static org.joda.time.format.DateTimeFormat.forPattern;
import static org.junit.Assert.assertTrue;

public class AuditDTOTest
{
  private static final long TEST_TIMESTAMP = 1549375691779L;

  private static final Pattern TIMESTAMP_PATTERN = Pattern.compile("(\\{\"timestamp\":\")([^\"]*)(\".*)");

  private AuditDTO underTest;

  @Test
  public void testToString_noData() {
    underTest = new AuditDTO();
    assertThat(underTest.toString(), is("{}"));
  }

  @Test
  public void testToString_withData() {
    // Setting Thread name is required because AuditData provides thread name for AuditDTO.
    Thread.currentThread().setName("quartz-7-thread-1");
    underTest = new AuditDTO(makeAuditData());

    Matcher timestampMatcher = TIMESTAMP_PATTERN.matcher(underTest.toString());
    assertTrue(timestampMatcher.matches());
    String timestamp = timestampMatcher.group(2);

    assertThat(parse(timestamp, forPattern("yyyy-MM-dd HH:mm:ss,SSSZ")).getMillis(), is(TEST_TIMESTAMP));

    assertThat(timestampMatcher.replaceAll("$1<TIMESTAMP>$3"), is(
        "{\"timestamp\":\"<TIMESTAMP>\",\"nodeId\":\"testnodeid\",\"initiator\":\"testinitiator\",\"domain\":\"testdomain\",\"type\":\"testtype\",\"context\":\"testcontext\",\"thread\":\"quartz-7-thread-1\",\"attributes\":{\"testattribute1\":\"testvalue1\",\"testattribute2\":\"testvalue2\"}}"));
  }

  private static AuditData makeAuditData() {
    AuditData data = new AuditData();
    data.setDomain("testdomain");
    data.setType("testtype");
    data.setContext("testcontext");
    data.setInitiator("testinitiator");
    data.setNodeId("testnodeid");
    data.setTimestamp(new Date(TEST_TIMESTAMP));
    data.getAttributes().put("testattribute1", "testvalue1");
    data.getAttributes().put("testattribute2", "testvalue2");
    return data;
  }
}
