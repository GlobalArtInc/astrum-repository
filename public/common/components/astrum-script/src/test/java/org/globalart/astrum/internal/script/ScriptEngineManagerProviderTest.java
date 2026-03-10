
package org.globalart.astrum.internal.script;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;

import javax.script.ScriptEngine;
import javax.script.ScriptEngineFactory;
import javax.script.ScriptEngineManager;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertSame;
import static org.junit.Assert.assertThrows;
import static org.mockito.Mockito.atLeastOnce;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.class)
public class ScriptEngineManagerProviderTest
{
  @Mock
  private ScriptEngineFactory factory;

  @Test
  public void constructorThrowsNullPointerExceptionWhenFactoriesIsNull() {
    assertThrows(NullPointerException.class, () -> new ScriptEngineManagerProvider(null));
  }

  @Test
  public void getObjectReturnsNonNullManagerWithEmptyFactories() {
    ScriptEngineManagerProvider provider = new ScriptEngineManagerProvider(new ArrayList<>());

    ScriptEngineManager manager = provider.getObject();

    assertNotNull(manager);
  }

  @Test
  public void getObjectRegistersInjectedFactoryByNameMimeTypeAndExtension() {
    when(factory.getNames()).thenReturn(Arrays.asList("test-engine"));
    when(factory.getMimeTypes()).thenReturn(Arrays.asList("application/x-test"));
    when(factory.getExtensions()).thenReturn(Arrays.asList("tst"));
    when(factory.getEngineName()).thenReturn("Test Engine");
    when(factory.getEngineVersion()).thenReturn("1.0");
    when(factory.getLanguageName()).thenReturn("test-lang");
    when(factory.getLanguageVersion()).thenReturn("2.0");

    ScriptEngine scriptEngine = mock(ScriptEngine.class);
    when(factory.getScriptEngine()).thenReturn(scriptEngine);

    ScriptEngineManagerProvider provider = new ScriptEngineManagerProvider(Arrays.asList(factory));

    ScriptEngineManager manager = provider.getObject();

    assertNotNull(manager);
    assertSame(scriptEngine, manager.getEngineByName("test-engine"));
    assertSame(scriptEngine, manager.getEngineByMimeType("application/x-test"));
    assertSame(scriptEngine, manager.getEngineByExtension("tst"));

    verify(factory, atLeastOnce()).getNames();
    verify(factory, atLeastOnce()).getMimeTypes();
    verify(factory, atLeastOnce()).getExtensions();
  }

  @Test
  public void getObjectHandlesFactoryWithNoNamesMimeTypesOrExtensions() {
    ScriptEngineFactory emptyFactory = mock(ScriptEngineFactory.class);
    when(emptyFactory.getNames()).thenReturn(Collections.emptyList());
    when(emptyFactory.getMimeTypes()).thenReturn(Collections.emptyList());
    when(emptyFactory.getExtensions()).thenReturn(Collections.emptyList());

    ScriptEngineManagerProvider provider = new ScriptEngineManagerProvider(Arrays.asList(emptyFactory));

    ScriptEngineManager manager = provider.getObject();

    assertNotNull(manager);

    // empty inner-loop branches: the factory is still consulted for each registration kind,
    // even though nothing ends up registered
    verify(emptyFactory, atLeastOnce()).getNames();
    verify(emptyFactory, atLeastOnce()).getMimeTypes();
    verify(emptyFactory, atLeastOnce()).getExtensions();
  }

  @Test
  public void getObjectRegistersEveryNameMimeTypeAndExtensionOfAFactory() {
    when(factory.getNames()).thenReturn(Arrays.asList("alpha", "beta"));
    when(factory.getMimeTypes()).thenReturn(Arrays.asList("application/x-alpha", "application/x-beta"));
    when(factory.getExtensions()).thenReturn(Arrays.asList("al", "be"));

    ScriptEngine scriptEngine = mock(ScriptEngine.class);
    when(factory.getScriptEngine()).thenReturn(scriptEngine);

    ScriptEngineManagerProvider provider = new ScriptEngineManagerProvider(Arrays.asList(factory));

    ScriptEngineManager manager = provider.getObject();

    // every element of each list must be registered, not just the first
    assertSame(scriptEngine, manager.getEngineByName("alpha"));
    assertSame(scriptEngine, manager.getEngineByName("beta"));
    assertSame(scriptEngine, manager.getEngineByMimeType("application/x-alpha"));
    assertSame(scriptEngine, manager.getEngineByMimeType("application/x-beta"));
    assertSame(scriptEngine, manager.getEngineByExtension("al"));
    assertSame(scriptEngine, manager.getEngineByExtension("be"));
  }

  @Test
  public void getObjectRegistersAllInjectedFactories() {
    ScriptEngineFactory firstFactory = mock(ScriptEngineFactory.class);
    when(firstFactory.getNames()).thenReturn(Arrays.asList("first"));
    when(firstFactory.getMimeTypes()).thenReturn(Collections.emptyList());
    when(firstFactory.getExtensions()).thenReturn(Collections.emptyList());
    ScriptEngine firstEngine = mock(ScriptEngine.class);
    when(firstFactory.getScriptEngine()).thenReturn(firstEngine);

    ScriptEngineFactory secondFactory = mock(ScriptEngineFactory.class);
    when(secondFactory.getNames()).thenReturn(Arrays.asList("second"));
    when(secondFactory.getMimeTypes()).thenReturn(Collections.emptyList());
    when(secondFactory.getExtensions()).thenReturn(Collections.emptyList());
    ScriptEngine secondEngine = mock(ScriptEngine.class);
    when(secondFactory.getScriptEngine()).thenReturn(secondEngine);

    ScriptEngineManagerProvider provider =
        new ScriptEngineManagerProvider(Arrays.asList(firstFactory, secondFactory));

    ScriptEngineManager manager = provider.getObject();

    // the outer loop must register every injected factory, not just the first
    assertSame(firstEngine, manager.getEngineByName("first"));
    assertSame(secondEngine, manager.getEngineByName("second"));
  }

  @Test
  public void getObjectTypeReturnsScriptEngineManagerClass() {
    ScriptEngineManagerProvider provider = new ScriptEngineManagerProvider(new ArrayList<>());

    assertEquals(ScriptEngineManager.class, provider.getObjectType());
  }
}
