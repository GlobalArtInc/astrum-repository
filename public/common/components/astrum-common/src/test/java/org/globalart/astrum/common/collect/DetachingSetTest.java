
package org.globalart.astrum.common.collect;

import java.util.Set;
import java.util.function.BooleanSupplier;
import java.util.function.Function;

import com.google.common.collect.ImmutableSet;
import org.junit.Before;
import org.junit.Test;
import org.mockito.InOrder;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;
import static org.mockito.AdditionalAnswers.returnsFirstArg;
import static org.mockito.ArgumentMatchers.isNotNull;
import static org.mockito.Mockito.inOrder;
import static org.mockito.Mockito.verifyNoMoreInteractions;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class DetachingSetTest

{
  @Mock
  private Set<String> backing;

  @Mock
  private BooleanSupplier allowDetach;

  @Mock
  private Function<String, String> detach;

  private DetachingSet<String> underTest;

  @Before
  public void setUp() {
    underTest = new DetachingSet<>(backing, allowDetach, detach);
  }

  @Test
  public void nonEscapingQueriesNeverDetach() {

    underTest.contains(null);
    underTest.containsAll(null);
    underTest.equals(null);
    underTest.hashCode();
    underTest.isEmpty();
    underTest.size();
    underTest.toString();

    InOrder inOrder = inOrder(backing);

    inOrder.verify(backing).contains(null);
    inOrder.verify(backing).containsAll(null);
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

    underTest.iterator();
    underTest.toArray();
    underTest.toArray(new String[0]);

    InOrder inOrder = inOrder(backing, allowDetach, detach);

    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).size();
    inOrder.verify(backing).forEach(isNotNull());
    // the rest happens on the detached set

    verifyNoMoreInteractions(backing, allowDetach, detach);
  }

  @Test
  public void mutationsTriggerDetach() {
    when(allowDetach.getAsBoolean()).thenReturn(true);

    underTest.add("");
    underTest.clear();
    underTest.remove("");

    InOrder inOrder = inOrder(backing, allowDetach, detach);

    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).size();
    inOrder.verify(backing).forEach(isNotNull());
    // the rest happens on the detached set

    verifyNoMoreInteractions(backing, allowDetach, detach);
  }

  @Test
  public void detachingCanBeDisallowed() {
    when(allowDetach.getAsBoolean()).thenReturn(false);

    underTest.add("");
    underTest.clear();
    underTest.remove("");
    underTest.iterator();
    underTest.toArray();
    underTest.toArray(new String[0]);

    InOrder inOrder = inOrder(backing, allowDetach, detach);

    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).add("");
    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).clear();
    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).remove("");
    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).iterator();
    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).toArray();
    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(backing).toArray(new String[0]);

    verifyNoMoreInteractions(backing, allowDetach, detach);
  }

  @Test
  public void simpleDetach() {
    Set<String> original = ImmutableSet.of("HELLO", "THERE");

    underTest = new DetachingSet<>(original, allowDetach, detach);

    when(allowDetach.getAsBoolean()).thenReturn(true);
    when(detach.apply(isNotNull())).thenAnswer(returnsFirstArg());

    assertThat(underTest.remove("THERE"), is(true));
    assertThat(underTest.add("WORLD"), is(true));
    assertThat(underTest, containsInAnyOrder("HELLO", "WORLD"));

    // original set contents should be unchanged
    assertThat(original, containsInAnyOrder("HELLO", "THERE"));

    InOrder inOrder = inOrder(allowDetach, detach);

    inOrder.verify(allowDetach).getAsBoolean();
    inOrder.verify(detach).apply("HELLO");
    inOrder.verify(detach).apply("THERE");

    verifyNoMoreInteractions(allowDetach, detach);
  }
}
