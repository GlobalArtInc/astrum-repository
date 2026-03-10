
package org.globalart.astrum.script;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.endsWith;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.Matchers.sameInstance;
import static org.hamcrest.Matchers.startsWith;
import static org.mockito.Mockito.mock;

public class ScriptEventTest
{
  @Test
  public void getScriptReturnsSameInstancePassedToConstructor() {
    Script script = mock(Script.class);

    ScriptEvent underTest = new ScriptEvent(script);

    assertThat(underTest.getScript(), sameInstance(script));
  }

  @Test
  public void getScriptReturnsNullWhenConstructedWithNull() {
    ScriptEvent underTest = new ScriptEvent(null);

    assertThat(underTest.getScript(), nullValue());
  }

  @Test
  public void toStringContainsSimpleClassNameAndScript() {
    Script script = mock(Script.class);

    ScriptEvent underTest = new ScriptEvent(script);

    assertThat(underTest.toString(), startsWith("ScriptEvent{script="));
    assertThat(underTest.toString(), endsWith("}"));
    assertThat(underTest.toString(), equalTo("ScriptEvent{script=" + script + "}"));
  }
}
