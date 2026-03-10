
package org.globalart.astrum.script;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.Matchers.sameInstance;
import static org.mockito.Mockito.mock;

public class ScriptDeletedEventTest
{
  @Test
  public void getScriptReturnsSameInstancePassedToConstructor() {
    Script script = mock(Script.class);

    ScriptDeletedEvent underTest = new ScriptDeletedEvent(script);

    assertThat(underTest.getScript(), sameInstance(script));
  }

  @Test
  public void getScriptReturnsNullWhenConstructedWithNull() {
    ScriptDeletedEvent underTest = new ScriptDeletedEvent(null);

    assertThat(underTest.getScript(), nullValue());
  }

  @Test
  public void eventIsInstanceOfScriptEvent() {
    Script script = mock(Script.class);

    ScriptDeletedEvent underTest = new ScriptDeletedEvent(script);

    assertThat(underTest, instanceOf(ScriptEvent.class));
  }

  @Test
  public void toStringUsesSubclassSimpleName() {
    Script script = mock(Script.class);

    ScriptDeletedEvent underTest = new ScriptDeletedEvent(script);

    assertThat(underTest.toString(), equalTo("ScriptDeletedEvent{script=" + script + "}"));
  }
}
