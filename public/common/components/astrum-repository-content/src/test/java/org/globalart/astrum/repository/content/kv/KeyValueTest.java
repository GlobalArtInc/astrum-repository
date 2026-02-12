
package org.globalart.astrum.repository.content.kv;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

public class KeyValueTest
{
  @Test
  public void testGettersAndSetters() {
    KeyValue underTest = new KeyValue();

    underTest.setCategory("metadata");
    underTest.setKey("version");
    underTest.setValue("1.0.0");
    OffsetDateTime now = OffsetDateTime.now(ZoneOffset.UTC);
    underTest.setCreated(now);

    assertThat(underTest.getCategory(), is("metadata"));
    assertThat(underTest.getKey(), is("version"));
    assertThat(underTest.getValue(), is("1.0.0"));
    assertThat(underTest.getCreated(), is(now));
  }

  @Test
  public void testDefaultValuesAreNull() {
    KeyValue underTest = new KeyValue();

    assertThat(underTest.getCategory(), is(nullValue()));
    assertThat(underTest.getKey(), is(nullValue()));
    assertThat(underTest.getValue(), is(nullValue()));
    assertThat(underTest.getCreated(), is(nullValue()));
  }

  @Test
  public void testContinuationTokenIsKey() {
    KeyValue underTest = new KeyValue();
    underTest.setKey("my-key");

    assertThat(underTest.nextContinuationToken(), is("my-key"));
  }
}
