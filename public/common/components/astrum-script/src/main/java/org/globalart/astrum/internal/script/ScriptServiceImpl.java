
package org.globalart.astrum.internal.script;

import java.util.List;
import java.util.Map;
import java.util.Map.Entry;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;
import javax.script.Bindings;
import javax.script.ScriptContext;
import javax.script.ScriptEngine;
import javax.script.ScriptEngineManager;
import javax.script.ScriptException;
import javax.script.SimpleScriptContext;

import org.globalart.astrum.common.app.GlobalComponentLookupHelper;
import org.globalart.astrum.common.script.ScriptApi;
import org.globalart.astrum.common.script.ScriptCleanupHandler;
import org.globalart.astrum.common.script.ScriptService;

import org.springframework.beans.factory.annotation.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import org.springframework.stereotype.Component;

/**
 * Default {@link ScriptService} implementation.
 *
 * @since 3.0
 */
@Component
public class ScriptServiceImpl
    implements ScriptService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String SCRIPT_CLEANUP_HANDLER = "scriptCleanupHandler";

  private final ScriptEngineManager engineManager;

  private final GlobalComponentLookupHelper lookupHelper;

  private final List<ScriptApi> scriptApis;

  private final ScriptCleanupHandler scriptCleanupHandler;

  private final boolean allowOnlyGroovy;

  @Autowired
  public ScriptServiceImpl(
      final ScriptEngineManager engineManager,
      final GlobalComponentLookupHelper lookupHelper,
      final List<ScriptApi> scriptApis,
      final ScriptCleanupHandler scriptCleanupHandler,
      @Value("${astrum.scripts.groovyOnly:true}") final boolean allowOnlyGroovy)
  {
    this.engineManager = checkNotNull(engineManager);
    this.lookupHelper = checkNotNull(lookupHelper);
    this.scriptApis = checkNotNull(scriptApis);
    this.scriptCleanupHandler = checkNotNull(scriptCleanupHandler);
    this.allowOnlyGroovy = allowOnlyGroovy;
  }

  @Override
  @Nonnull
  public ScriptEngine engineForLanguage(final String language) {
    validateLanguage(language);

    log.trace("Resolving engine for language: {}", language);
    ScriptEngine engine = engineManager.getEngineByName(language);
    checkState(engine != null, "Missing engine for language: %s", language);
    log.trace("Engine: {}", engine);
    return engine;
  }

  @Override
  public void applyDefaultBindings(final Bindings bindings) {
    checkNotNull(bindings);

    // TODO: Could potentially expose these in GLOBAL_SCOPE bindings?
    // TODO: Consider options to allow scripts more easily become injected aware?

    bindings.put("container", lookupHelper);
    bindings.put(SCRIPT_CLEANUP_HANDLER, scriptCleanupHandler);
    for (ScriptApi scriptApi : scriptApis) {
      bindings.put(scriptApi.getName(), scriptApi);
    }
  }

  @Nonnull
  @Override
  public ScriptContext createContext(final String language) {
    ScriptContext context = new SimpleScriptContext();
    context.setBindings(engineForLanguage(language).createBindings(), ScriptContext.ENGINE_SCOPE);
    return context;
  }

  @Override
  public void customizeBindings(
      final ScriptContext context,
      final int scope,
      final Map<String, Object> customizations)
  {
    Bindings bindings = context.getBindings(scope);
    applyDefaultBindings(bindings);
    for (Entry<String, Object> entry : customizations.entrySet()) {
      bindings.put(entry.getKey(), entry.getValue());
    }
  }

  @Override
  public void customizeBindings(final ScriptContext context, final Map<String, Object> customizations) {
    customizeBindings(context, ScriptContext.ENGINE_SCOPE, customizations);
  }

  @Override
  public Object eval(final String language, final String script, final ScriptContext context) throws ScriptException {
    return engineForLanguage(checkNotNull(language)).eval(checkNotNull(script), checkNotNull(context));
  }

  @Override
  public Object eval(
      final String language,
      final String script,
      final Map<String, Object> customBindings) throws ScriptException
  {
    ScriptContext context = createContext(language);
    customizeBindings(context, customBindings);
    return eval(language, script, context);
  }

  private void validateLanguage(final String language) {
    checkNotNull(language);

    if (allowOnlyGroovy && !language.equals(ScriptEngineManagerProvider.DEFAULT_LANGUAGE)) {
      throw new IllegalScriptLanguageException("Language: " + language + " is not allowed");
    }
  }
}
