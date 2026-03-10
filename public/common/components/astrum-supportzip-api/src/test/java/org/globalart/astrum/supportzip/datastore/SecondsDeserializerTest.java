
package org.globalart.astrum.supportzip.datastore;

import java.io.IOException;
import java.util.concurrent.TimeUnit;

import org.globalart.astrum.common.time.Time;

import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.module.SimpleModule;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link SecondsDeserializer}
 */
public class SecondsDeserializerTest
{
  private ObjectMapper mapperWithDeserializer() {
    SimpleModule module = new SimpleModule();
    module.addDeserializer(Time.class, new SecondsDeserializer());
    return new ObjectMapper().registerModule(module);
  }

  @Test
  public void testConstructorRegistersTimeAsHandledType() {
    SecondsDeserializer deserializer = new SecondsDeserializer();
    // StdDeserializer.handledType() is declared as Class<?>, so compare by name to assert the exact type
    assertThat(deserializer.handledType().getName(), is(Time.class.getName()));
  }

  @Test
  public void testDeserializeReadsLongAndReturnsTimeInSeconds() throws IOException {
    JsonParser parser = mock(JsonParser.class);
    DeserializationContext context = mock(DeserializationContext.class);
    when(parser.readValueAs(Long.class)).thenReturn(42L);

    Time time = new SecondsDeserializer().deserialize(parser, context);

    assertThat(time, is(Time.seconds(42)));
    assertThat(time.value(), is(42L));
    assertThat(time.unit(), is(TimeUnit.SECONDS));
    assertThat(time.toSecondsI(), is(42));
  }

  @Test
  public void testDeserializeZero() throws IOException {
    JsonParser parser = mock(JsonParser.class);
    DeserializationContext context = mock(DeserializationContext.class);
    when(parser.readValueAs(Long.class)).thenReturn(0L);

    Time time = new SecondsDeserializer().deserialize(parser, context);

    assertThat(time, is(Time.seconds(0)));
    assertThat(time.unit(), is(TimeUnit.SECONDS));
    assertThat(time.toSecondsI(), is(0));
  }

  @Test
  public void testDeserializeNegativeValue() throws IOException {
    JsonParser parser = mock(JsonParser.class);
    DeserializationContext context = mock(DeserializationContext.class);
    when(parser.readValueAs(Long.class)).thenReturn(-5L);

    Time time = new SecondsDeserializer().deserialize(parser, context);

    assertThat(time, is(Time.seconds(-5)));
    assertThat(time.toSecondsI(), is(-5));
  }

  @Test
  public void testDeserializeValueExceedingIntRangePreservedAsLong() throws IOException {
    // a value larger than Integer.MAX_VALUE must survive as a long (parser reads Long, not Integer)
    long largeValue = Integer.MAX_VALUE + 1L;
    JsonParser parser = mock(JsonParser.class);
    DeserializationContext context = mock(DeserializationContext.class);
    when(parser.readValueAs(Long.class)).thenReturn(largeValue);

    Time time = new SecondsDeserializer().deserialize(parser, context);

    assertThat(time, is(Time.seconds(largeValue)));
    assertThat(time.value(), is(largeValue));
    assertThat(time.toSeconds(), is(largeValue));
  }

  @Test(expected = NullPointerException.class)
  public void testDeserializeNullValueThrowsNullPointerException() throws IOException {
    // a null token unboxes to long inside Time.seconds(...) and must fail fast
    JsonParser parser = mock(JsonParser.class);
    DeserializationContext context = mock(DeserializationContext.class);
    when(parser.readValueAs(Long.class)).thenReturn(null);

    new SecondsDeserializer().deserialize(parser, context);
  }

  @Test
  public void testRoundTripViaObjectMapper() throws IOException {
    Time time = mapperWithDeserializer().readValue("42", Time.class);

    assertThat(time, is(Time.seconds(42)));
    assertThat(time.unit(), is(TimeUnit.SECONDS));
    assertThat(time.toSecondsI(), is(42));
  }

  @Test
  public void testRoundTripLargeValueViaObjectMapper() throws IOException {
    long largeValue = Integer.MAX_VALUE + 1L;
    Time time = mapperWithDeserializer().readValue(Long.toString(largeValue), Time.class);

    assertThat(time, is(Time.seconds(largeValue)));
    assertThat(time.toSeconds(), is(largeValue));
  }
}
