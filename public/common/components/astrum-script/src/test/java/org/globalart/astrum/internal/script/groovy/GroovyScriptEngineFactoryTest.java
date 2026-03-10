
package org.globalart.astrum.internal.script.groovy;

import javax.script.Bindings;
import javax.script.ScriptContext;
import javax.script.ScriptEngine;
import javax.script.ScriptException;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.script.ScriptCleanupHandler;
import org.globalart.astrum.internal.script.ScriptServiceImpl;
import org.globalart.astrum.internal.script.ScriptTask;
import org.globalart.astrum.scheduling.TaskSupport;

import groovy.lang.Binding;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertSame;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.class)
public class GroovyScriptEngineFactoryTest
{

  @Mock
  private ApplicationDirectories applicationDirectories;

  @Mock
  private ScriptCleanupHandler scriptCleanupHandler;

  @Test
  public void itWillCallTheCleanupHelperAfterExecutingTheScript() throws ScriptException {
    GroovyScriptEngineFactory factory = new GroovyScriptEngineFactory(applicationDirectories);
    ScriptEngine engine = factory.getScriptEngine();
    Bindings bindings = engine.getBindings(ScriptContext.ENGINE_SCOPE);
    bindings.put(ScriptServiceImpl.SCRIPT_CLEANUP_HANDLER, scriptCleanupHandler);

    Object result = engine.eval("return 1", bindings);
    assertEquals(1, result);
    verify(scriptCleanupHandler, times(1)).cleanup(any());
  }

  @Test
  public void itWillCallTheCleanupHelperAfterExecutingTheScriptEvenWhenAnExceptionIsThrown() {
    GroovyScriptEngineFactory factory = new GroovyScriptEngineFactory(applicationDirectories);
    ScriptEngine engine = factory.getScriptEngine();
    Bindings bindings = engine.getBindings(ScriptContext.ENGINE_SCOPE);
    bindings.put(ScriptServiceImpl.SCRIPT_CLEANUP_HANDLER, scriptCleanupHandler);

    assertThrows(ScriptException.class, () -> {
      engine.eval("throw new RuntimeException(\"bad\")", bindings);
    });

    verify(scriptCleanupHandler, times(1)).cleanup(any());
  }

  @Test
  public void itWillCreateANamedContextForTheExecutorOfTheGroovyScriptEngine() {
    GroovyScriptEngineFactory factory = new GroovyScriptEngineFactory(applicationDirectories);
    ScriptEngine engine = factory.getScriptEngine();
    Binding binding = new Binding();
    binding.setVariable("task", null);
    binding.setVariable("scriptName", "testScript");

    String expectedContext = "Script 'testScript'";
    String actualContext = GroovyScriptEngineFactory.getContext(binding);

    assertEquals(expectedContext, actualContext);

    ScriptTask task = mock(ScriptTask.class);
    when(task.getName()).thenReturn("myTask");
    binding.setVariable("task", task);
    binding.setVariable("scriptName", null);

    expectedContext = "Task 'myTask'";
    actualContext = GroovyScriptEngineFactory.getContext(binding);

    assertEquals(expectedContext, actualContext);

    binding.setVariable("task", null);
    binding.setVariable("scriptName", null);

    expectedContext = "An unknown script";
    actualContext = GroovyScriptEngineFactory.getContext(binding);

    assertEquals(expectedContext, actualContext);

    TaskSupport taskSupport = mock(TaskSupport.class);
    binding.setVariable("task", taskSupport);
    binding.setVariable("scriptName", null);

    expectedContext = "An unknown script";
    actualContext = GroovyScriptEngineFactory.getContext(binding);

    assertEquals(expectedContext, actualContext);
  }

  @Test
  public void getScriptEngineReturnsTheSameCachedInstanceOnSubsequentCalls() {
    GroovyScriptEngineFactory factory = new GroovyScriptEngineFactory(applicationDirectories);

    ScriptEngine first = factory.getScriptEngine();
    ScriptEngine second = factory.getScriptEngine();

    assertSame(first, second);
  }

  @Test
  public void itWillNotCallCleanupWhenTheHandlerBindingIsNotAScriptCleanupHandler() throws ScriptException {
    GroovyScriptEngineFactory factory = new GroovyScriptEngineFactory(applicationDirectories);
    ScriptEngine engine = factory.getScriptEngine();
    Bindings bindings = engine.getBindings(ScriptContext.ENGINE_SCOPE);
    bindings.put(ScriptServiceImpl.SCRIPT_CLEANUP_HANDLER, "not-a-cleanup-handler");

    Object result = engine.eval("return 2", bindings);

    assertEquals(2, result);
    verify(scriptCleanupHandler, never()).cleanup(any());
  }
}
