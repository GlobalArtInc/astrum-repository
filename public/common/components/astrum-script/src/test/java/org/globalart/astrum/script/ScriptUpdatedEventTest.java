
package org.globalart.astrum.script;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.Matchers.sameInstance;
import static org.mockito.Mockito.mock;

public class ScriptUpdatedEventTest
{
  @Test
  public void getScriptReturnsSameInstancePassedToConstructor() {
    Script script = mock(Script.class);

    ScriptUpdatedEvent underTest = new ScriptUpdatedEvent(script);

    assertThat(underTest.getScript(), sameInstance(script));
  }

  @Test
  public void getScriptReturnsNullWhenConstructedWithNull() {
    ScriptUpdatedEvent underTest = new ScriptUpdatedEvent(null);

    assertThat(underTest.getScript(), nullValue());
  }

  @Test
  public void eventIsInstanceOfScriptEvent() {
    Script script = mock(Script.class);

    ScriptUpdatedEvent underTest = new ScriptUpdatedEvent(script);

    assertThat(underTest, instanceOf(ScriptEvent.class));
  }

  @Test
  public void toStringUsesSubclassSimpleName() {
    Script script = mock(Script.class);

    ScriptUpdatedEvent underTest = new ScriptUpdatedEvent(script);

    assertThat(underTest.toString(), equalTo("ScriptUpdatedEvent{script=" + script + "}"));
  }
}
