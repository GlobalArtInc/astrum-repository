
package org.globalart.astrum.internal.script;

import java.util.Map;

import javax.script.ScriptException;

import org.globalart.astrum.common.script.ScriptService;
import org.globalart.astrum.scheduling.TaskConfiguration;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.slf4j.Logger;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.sameInstance;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.anyMap;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.class)
public class ScriptTaskTest
{
  private static final String LANGUAGE = "groovy";

  private static final String SOURCE = "log.info('hello world')";

  @Mock
  private ScriptService scriptService;

  private ScriptTask underTest;

  @Before
  public void setUp() {
    underTest = new ScriptTask(scriptService);
  }

  @Test
  public void constructorRejectsNullScriptService() {
    assertThrows(NullPointerException.class, () -> new ScriptTask(null));
  }

  @Test
  public void getMessageReturnsExecuteScript() {
    assertThat(underTest.getMessage(), is("Execute script"));
  }

  @Test
  public void executeEvaluatesConfiguredScriptAndReturnsResult() throws Exception {
    Object expected = new Object();
    underTest.configure(newConfiguration(LANGUAGE, SOURCE));
    when(scriptService.eval(eq(LANGUAGE), eq(SOURCE), anyMap())).thenReturn(expected);

    Object result = underTest.execute();

    assertThat(result, is(expected));
    verify(scriptService).eval(eq(LANGUAGE), eq(SOURCE), anyMap());
  }

  @Test
  public void executePassesLogAndTaskBindingsToScriptService() throws Exception {
    underTest.configure(newConfiguration(LANGUAGE, SOURCE));
    when(scriptService.eval(eq(LANGUAGE), eq(SOURCE), anyMap())).thenReturn(null);

    underTest.execute();

    ArgumentCaptor<Map<String, Object>> bindingsCaptor = ArgumentCaptor.captor();
    verify(scriptService).eval(eq(LANGUAGE), eq(SOURCE), bindingsCaptor.capture());

    Map<String, Object> bindings = bindingsCaptor.getValue();
    assertThat(bindings.size(), is(2));
    assertThat(bindings.containsKey("log"), is(true));
    assertThat(bindings.containsKey("task"), is(true));

    Object logBinding = bindings.get("log");
    assertThat(logBinding, is(instanceOf(Logger.class)));
    assertThat(((Logger) logBinding).getName(), is(ScriptTask.class.getName()));

    assertThat(bindings.get("task"), is(sameInstance((Object) underTest)));
  }

  @Test
  public void executePassesConfiguredLanguageAndSourceToScriptService() throws Exception {
    underTest.configure(newConfiguration(LANGUAGE, SOURCE));
    when(scriptService.eval(anyString(), anyString(), anyMap())).thenReturn(null);

    underTest.execute();

    ArgumentCaptor<String> languageCaptor = ArgumentCaptor.forClass(String.class);
    ArgumentCaptor<String> sourceCaptor = ArgumentCaptor.forClass(String.class);
    verify(scriptService).eval(languageCaptor.capture(), sourceCaptor.capture(), anyMap());

    assertThat(languageCaptor.getValue(), is(LANGUAGE));
    assertThat(sourceCaptor.getValue(), is(SOURCE));
  }

  @Test
  public void executePropagatesScriptServiceException() throws Exception {
    ScriptException failure = new ScriptException("boom");
    underTest.configure(newConfiguration(LANGUAGE, SOURCE));
    when(scriptService.eval(eq(LANGUAGE), eq(SOURCE), anyMap())).thenThrow(failure);

    ScriptException thrown = assertThrows(ScriptException.class, () -> underTest.execute());

    assertThat(thrown, is(sameInstance(failure)));
  }

  private TaskConfiguration newConfiguration(final String language, final String source) {
    TaskConfiguration configuration = new TaskConfiguration();
    configuration.setId("test-id");
    configuration.setTypeId(ScriptTaskDescriptor.TYPE_ID);
    configuration.setString(ScriptTaskDescriptor.LANGUAGE, language);
    configuration.setString(ScriptTaskDescriptor.SOURCE, source);
    return configuration;
  }
}
