
package org.globalart.astrum.common.collect;

import java.util.Map;
import java.util.function.BooleanSupplier;
import java.util.function.Function;

import com.google.common.collect.ImmutableMap;
import org.junit.Before;
import org.junit.Test;
import org.mockito.InOrder;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasEntry;
import static org.hamcrest.Matchers.is;
import static org.mockito.AdditionalAnswers.returnsFirstArg;
import static org.mockito.ArgumentMatchers.isNotNull;
import static org.mockito.Mockito.inOrder;
import static org.mockito.Mockito.verifyNoMoreInteractions;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class DetachingMapTest

{
  @Mock
  private Map<String, String> backing;

  @Mock
  private BooleanSupplier allowDetach;

  @Mock
  private Function<String, String> detach;

  private DetachingMap<String, String> underTest;

  @Before
  public void setUp() {
    underTest = new DetachingMap<>(backing, allowDetach, detach);
  }

  @Test
  public void nonEscapingQueriesNeverDetach() {

    underTest.containsKey(null);
    underTest.containsValue(null);
    underTest.equals(null);
    underTest.hashCode();
    underTest.isEmpty();
    underTest.size();
    underTest.toString();

    InOrder inOrder = inOrder(backing);

    inOrder.verify(backing).containsKey(null);
    inOrder.verify(backing).containsValue(null);
    // Mockito ignores equals
    // Mockito ignores hashCode
    inOrder.verify(backing).isEmpty();
    inOrder.verify(backing).size();
    // Mockito ignores toString

    verifyNoMoreInteractions(backing);

    verifyNoInteractions(allowDetach, detach);
  }

  @Test
  public void escapingQueriesTriggerDetach() {
    when(allowDetach.getAsBoolean()).thenReturn(true);

    underTest.keySet();
    underTest.entrySet();
    underTest.values();

    InOrder inOrder = inOrder(backing, allowDetach, detach);

    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).size();
    inOrder.verify(backing).entrySet();
    // the rest happens on the detached map

    verifyNoMoreInteractions(backing, allowDetach, detach);
  }

  @Test
  public void mutationsTriggerDetach() {
    when(allowDetach.getAsBoolean()).thenReturn(true);

    underTest.put("foo", "bar");
    underTest.clear();
    underTest.remove("foo");

    InOrder inOrder = inOrder(backing, allowDetach, detach);

    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).size();
    inOrder.verify(backing).entrySet();
    // the rest happens on the detached map

    verifyNoMoreInteractions(backing, allowDetach, detach);
  }

  @Test
  public void detachingCanBeDisallowed() {
    when(allowDetach.getAsBoolean()).thenReturn(false);

    underTest.put("foo", "bar");
    underTest.clear();
    underTest.remove("foo");
    underTest.keySet();
    underTest.entrySet();
    underTest.values();

    InOrder inOrder = inOrder(backing, allowDetach, detach);

    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).put("foo", "bar");
    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).clear();
    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).remove("foo");
    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).keySet();
    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).entrySet();
    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).values();

    verifyNoMoreInteractions(backing, allowDetach, detach);
  }

  @Test
  public void simpleDetach() {
    Map<String, String> original = ImmutableMap.of("1", "I", "2", "two", "3", "III");

    underTest = new DetachingMap<>(original, allowDetach, detach);

    when(allowDetach.getAsBoolean()).thenReturn(true);
    when(detach.apply(isNotNull())).thenAnswer(returnsFirstArg());

    assertThat(underTest.put("2", "II"), is("two"));
    assertThat(underTest, hasEntry("2", "II"));

    // original map contents should be unchanged
    assertThat(original, hasEntry("2", "two"));

    InOrder inOrder = inOrder(allowDetach, detach);

    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(detach).apply("I");
    inOrder.verify(detach).apply("two");
    inOrder.verify(detach).apply("III");

    verifyNoMoreInteractions(allowDetach, detach);
  }
}
