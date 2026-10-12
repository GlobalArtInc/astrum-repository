package org.globalart.astrum.repository.rubygems.internal.gem;

import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Date;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TimeZone;

import javax.annotation.Nullable;

import org.yaml.snakeyaml.DumperOptions;
import org.yaml.snakeyaml.LoaderOptions;
import org.yaml.snakeyaml.Yaml;
import org.yaml.snakeyaml.constructor.AbstractConstruct;
import org.yaml.snakeyaml.constructor.SafeConstructor;
import org.yaml.snakeyaml.error.YAMLException;
import org.yaml.snakeyaml.nodes.Node;
import org.yaml.snakeyaml.nodes.Tag;
import org.yaml.snakeyaml.representer.Representer;
import org.yaml.snakeyaml.resolver.Resolver;

public final class GemSpecParser
{
  private static final int CODE_POINT_LIMIT = 32 * 1024 * 1024;

  private GemSpecParser() {
  }

  public static GemSpec parse(final InputStream yaml) {
    Object document;
    try {
      document = yaml().load(new InputStreamReader(yaml, StandardCharsets.UTF_8));
    }
    catch (YAMLException e) {
      throw new IllegalArgumentException("Invalid gem specification: " + e.getMessage(), e);
    }
    if (!(document instanceof Map<?, ?> spec)) {
      throw new IllegalArgumentException("Invalid gem specification: not a mapping");
    }
    return toSpec(spec);
  }

  static GemSpec toSpec(final Map<?, ?> spec) {
    String name = text(spec.get("name"));
    String version = version(spec.get("version"));
    if (!GemCoordinates.isValidName(name)) {
      throw new IllegalArgumentException("Invalid gem name: " + name);
    }
    if (!GemVersion.isValid(version)) {
      throw new IllegalArgumentException("Invalid gem version: " + version);
    }
    String platform = platform(spec.get("platform"));
    if (!GemPlatform.isValid(platform)) {
      throw new IllegalArgumentException("Invalid gem platform: " + platform);
    }
    List<String> licenses = strings(spec.get("licenses"));
    if (licenses.isEmpty()) {
      licenses = strings(spec.get("license"));
    }
    List<GemDependency> dependencies = new ArrayList<>();
    if (spec.get("dependencies") instanceof List<?> deps) {
      for (Object item : deps) {
        if (item instanceof Map<?, ?> dep) {
          String depName = text(dep.get("name"));
          if (depName == null) {
            continue;
          }
          Object requirement = dep.get("requirement") != null ? dep.get("requirement") : dep.get("version_requirements");
          String type = text(dep.get("type"));
          type = type == null ? GemDependency.RUNTIME : type.replaceFirst("^:", "");
          dependencies.add(new GemDependency(depName, type, requirements(requirement)));
        }
      }
    }
    String specificationVersion = text(spec.get("specification_version"));
    return new GemSpec(
        name,
        version.strip(),
        platform,
        text(spec.get("summary")),
        text(spec.get("description")),
        strings(spec.get("authors")),
        strings(spec.get("email")),
        text(spec.get("homepage")),
        licenses,
        metadata(spec.get("metadata")),
        dependencies,
        requirements(spec.get("required_ruby_version")),
        requirements(spec.get("required_rubygems_version")),
        text(spec.get("rubygems_version")),
        specificationVersion != null && specificationVersion.matches("-?\\d{1,3}")
            ? Integer.parseInt(specificationVersion) : 4,
        date(spec.get("date")));
  }

  private static Yaml yaml() {
    LoaderOptions options = new LoaderOptions();
    options.setAllowDuplicateKeys(true);
    options.setCodePointLimit(CODE_POINT_LIMIT);
    options.setMaxAliasesForCollections(200);
    options.setTagInspector(tag -> true);
    DumperOptions dumperOptions = new DumperOptions();
    return new Yaml(new RubyObjectConstructor(options), new Representer(dumperOptions), dumperOptions, options,
        new PlainResolver());
  }

  @Nullable
  static String version(@Nullable final Object value) {
    if (value instanceof Map<?, ?> map) {
      return text(map.get("version"));
    }
    return text(value);
  }

  static String platform(@Nullable final Object value) {
    if (value instanceof Map<?, ?> map) {
      List<String> parts = new ArrayList<>();
      for (String key : List.of("cpu", "os", "version")) {
        String part = text(map.get(key));
        if (part != null && !part.isBlank()) {
          parts.add(part);
        }
      }
      return GemPlatform.normalize(String.join("-", parts));
    }
    return GemPlatform.normalize(text(value));
  }

  static List<String> requirements(@Nullable final Object value) {
    List<String> result = new ArrayList<>();
    Object requirements = value instanceof Map<?, ?> map ? map.get("requirements") : value;
    if (requirements instanceof List<?> list) {
      for (Object item : list) {
        if (item instanceof List<?> pair && pair.size() == 2) {
          String operator = text(pair.get(0));
          String version = version(pair.get(1));
          if (operator != null && version != null) {
            result.add(operator.strip() + " " + version.strip());
          }
        }
        else if (item instanceof String text && !text.isBlank()) {
          result.add(GemRequirements.normalize(text));
        }
      }
    }
    else if (requirements instanceof String text && !text.isBlank()) {
      for (String part : text.split(",")) {
        result.add(GemRequirements.normalize(part));
      }
    }
    return result;
  }

  private static Map<String, String> metadata(@Nullable final Object value) {
    Map<String, String> metadata = new LinkedHashMap<>();
    if (value instanceof Map<?, ?> map) {
      map.forEach((key, item) -> {
        String text = text(item);
        if (key != null && text != null) {
          metadata.put(key.toString(), text);
        }
      });
    }
    return metadata;
  }

  @Nullable
  private static String date(@Nullable final Object value) {
    if (value instanceof Date date) {
      SimpleDateFormat format = new SimpleDateFormat("yyyy-MM-dd");
      format.setTimeZone(TimeZone.getTimeZone("UTC"));
      return format.format(date);
    }
    String text = text(value);
    if (text != null && text.matches("\\d{4}-\\d{2}-\\d{2}.*")) {
      return text.substring(0, 10);
    }
    return null;
  }

  private static List<String> strings(@Nullable final Object value) {
    List<String> result = new ArrayList<>();
    if (value instanceof List<?> list) {
      for (Object item : list) {
        String text = text(item);
        if (text != null && !text.isBlank()) {
          result.add(text);
        }
      }
    }
    else {
      String text = text(value);
      if (text != null && !text.isBlank()) {
        result.add(text);
      }
    }
    return result;
  }

  @Nullable
  static String text(@Nullable final Object value) {
    if (value == null || value instanceof Map || value instanceof List) {
      return null;
    }
    if (value instanceof byte[] bytes) {
      return new String(bytes, StandardCharsets.UTF_8);
    }
    return value.toString();
  }

  private static final class RubyObjectConstructor
      extends SafeConstructor
  {
    private RubyObjectConstructor(final LoaderOptions options) {
      super(options);
      this.yamlConstructors.put(null, new ConstructByNodeKind());
      this.yamlConstructors.put(new Tag("!binary"), this.yamlConstructors.get(Tag.BINARY));
    }

    private final class ConstructByNodeKind
        extends AbstractConstruct
    {
      @Override
      public Object construct(final Node node) {
        Tag tag = switch (node.getNodeId()) {
          case mapping -> Tag.MAP;
          case sequence -> Tag.SEQ;
          default -> Tag.STR;
        };
        node.setTag(tag);
        return yamlConstructors.get(tag).construct(node);
      }
    }
  }

  private static final class PlainResolver
      extends Resolver
  {
    @Override
    protected void addImplicitResolvers() {
      addImplicitResolver(Tag.NULL, NULL, "~nN\0");
      addImplicitResolver(Tag.NULL, EMPTY, null);
      addImplicitResolver(Tag.BOOL, BOOL, "yYnNtTfFoO");
    }
  }
}
