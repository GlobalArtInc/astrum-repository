
package org.globalart.astrum.internal.script.groovy;

import javax.script.ScriptEngine;

import groovy.lang.GroovyClassLoader;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Groovy {@link ScriptEngine}.
 *
 * @since 3.0
 */
public class GroovyScriptEngine
    extends org.codehaus.groovy.jsr223.GroovyScriptEngineImpl
{
  private static final Logger log = LoggerFactory.getLogger(GroovyScriptEngine.class);

  // FIXME: Sort out how we can better avoid leaking generated classes
  // FIXME: It appears the default impl will retain generated classes, even for evaluation calls (not just for compiled)
  // FIXME: Probably needs to use GroovyClassLoader.parseClass(GroovyCodeSource codeSource, boolean shouldCacheSource)

  public GroovyScriptEngine(final GroovyClassLoader classLoader) {
    super(classLoader);
  }

  // TODO: Sub-class here to add customized support to handle class cache
}
