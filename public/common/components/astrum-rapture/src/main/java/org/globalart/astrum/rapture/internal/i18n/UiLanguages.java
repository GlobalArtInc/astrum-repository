package org.globalart.astrum.rapture.internal.i18n;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.Reader;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Enumeration;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import java.util.TreeSet;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import jakarta.servlet.http.Cookie;
import jakarta.servlet.http.HttpServletRequest;

import com.google.gson.Gson;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.core.io.Resource;
import org.springframework.core.io.support.PathMatchingResourcePatternResolver;
import org.springframework.stereotype.Component;

/**
 * The languages the UI can be shown in. English is built in; another language is available when some module ships a
 * dictionary for it at {@code astrum-i18n/<dictionary>/<language>.json}. The language of a request is the one in the
 * {@value #COOKIE} cookie, else the first available one the browser accepts, else English.
 */
@Component
public class UiLanguages
{
  public static final String DEFAULT = "en";

  public static final String COOKIE = "astrum-language";

  static final String DICTIONARIES = "classpath*:astrum-i18n/*/*.json";

  static final String EXT_LOCALE = "astrum-i18n/ext-locale-%s.js";

  private static final Pattern DICTIONARY_PATH = Pattern.compile(".*/astrum-i18n/([^/]+)/([A-Za-z_]+)\\.json$");

  private static final Logger log = LoggerFactory.getLogger(UiLanguages.class);

  private final Map<String, JsonObject> dictionaries = new TreeMap<>();

  private final ClassLoader classLoader;

  public UiLanguages() {
    this(UiLanguages.class.getClassLoader());
  }

  UiLanguages(final ClassLoader classLoader) {
    this.classLoader = classLoader;
    try {
      for (Resource resource : new PathMatchingResourcePatternResolver(classLoader).getResources(DICTIONARIES)) {
        load(resource);
      }
    }
    catch (IOException e) {
      log.warn("Could not list the UI dictionaries", e);
    }
    log.info("UI languages: {}", available());
  }

  private void load(final Resource resource) {
    Matcher matcher;
    try {
      matcher = DICTIONARY_PATH.matcher(resource.getURL().toString());
    }
    catch (IOException e) {
      log.warn("Ignoring UI dictionary {}", resource, e);
      return;
    }
    if (!matcher.matches() || DEFAULT.equals(matcher.group(2))) {
      return;
    }
    String dictionary = matcher.group(1);
    String language = matcher.group(2);
    try (Reader reader = new InputStreamReader(resource.getInputStream(), StandardCharsets.UTF_8)) {
      JsonElement content = JsonParser.parseReader(reader);
      if (!content.isJsonObject()) {
        log.warn("Ignoring UI dictionary {}: not a JSON object", resource);
        return;
      }
      JsonObject byDictionary = dictionaries.computeIfAbsent(language, key -> new JsonObject());
      JsonObject entries = byDictionary.has(dictionary) ? byDictionary.getAsJsonObject(dictionary) : new JsonObject();
      content.getAsJsonObject().entrySet().forEach(entry -> entries.add(entry.getKey(), entry.getValue()));
      byDictionary.add(dictionary, entries);
    }
    catch (IOException | RuntimeException e) {
      log.warn("Ignoring UI dictionary {}", resource, e);
    }
  }

  /**
   * @return the available languages, English first
   */
  public List<String> available() {
    List<String> languages = new ArrayList<>();
    languages.add(DEFAULT);
    languages.addAll(new TreeSet<>(dictionaries.keySet()));
    return languages;
  }

  public boolean isAvailable(final String language) {
    return language != null && (DEFAULT.equals(language) || dictionaries.containsKey(language));
  }

  public String resolve(final HttpServletRequest request) {
    if (request == null) {
      return DEFAULT;
    }
    Cookie[] cookies = request.getCookies();
    if (cookies != null) {
      for (Cookie cookie : cookies) {
        if (COOKIE.equals(cookie.getName()) && isAvailable(cookie.getValue())) {
          return cookie.getValue();
        }
      }
    }
    Enumeration<Locale> locales = request.getLocales();
    while (locales != null && locales.hasMoreElements()) {
      String language = locales.nextElement().getLanguage();
      if (isAvailable(language)) {
        return language;
      }
    }
    return DEFAULT;
  }

  /**
   * The name of a language in that language, e.g. {@code Русский}.
   */
  public static String nativeName(final String language) {
    Locale locale = Locale.forLanguageTag(language.replace('_', '-'));
    String name = locale.getDisplayLanguage(locale);
    if (name.isEmpty()) {
      return language;
    }
    return name.substring(0, 1).toUpperCase(locale) + name.substring(1);
  }

  /**
   * The script that hands the dictionaries of a language to the UI as {@code window.AS_I18N}, followed by the
   * translations of the ExtJS framework itself when there are any.
   */
  public byte[] script(final String language) {
    JsonObject i18n = new JsonObject();
    i18n.addProperty("language", language);
    i18n.add("dictionaries", dictionaries.getOrDefault(language, new JsonObject()));
    StringBuilder script = new StringBuilder("window.AS_I18N = ").append(new Gson().toJson(i18n)).append(";\n");
    try (InputStream extLocale = classLoader.getResourceAsStream(String.format(EXT_LOCALE, language))) {
      if (extLocale != null) {
        script.append(new String(extLocale.readAllBytes(), StandardCharsets.UTF_8));
      }
    }
    catch (IOException e) {
      log.warn("Could not read the ExtJS translations for {}", language, e);
    }
    return script.toString().getBytes(StandardCharsets.UTF_8);
  }

  Set<String> dictionaryNames(final String language) {
    return dictionaries.getOrDefault(language, new JsonObject()).keySet();
  }
}
