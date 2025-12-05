
package org.globalart.astrum.internal.atlas.customizers;

import java.io.File;
import java.io.FileWriter;
import java.io.IOException;
import java.util.ArrayList;
import java.util.Collection;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;
import javax.management.AttributeNotFoundException;
import javax.management.InstanceNotFoundException;
import javax.management.IntrospectionException;
import javax.management.MBeanException;
import javax.management.MBeanInfo;
import javax.management.MBeanServer;
import javax.management.MalformedObjectNameException;
import javax.management.ObjectName;
import javax.management.ReflectionException;
import javax.management.RuntimeMBeanException;
import javax.management.openmbean.CompositeData;
import javax.management.openmbean.TabularData;

import org.globalart.astrum.supportzip.GeneratedContentSourceSupport;
import org.globalart.astrum.supportzip.SupportBundle;
import org.globalart.astrum.supportzip.SupportBundleCustomizer;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.google.common.annotations.VisibleForTesting;
import org.springframework.beans.factory.annotation.Qualifier;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Arrays.asList;
import static java.util.Arrays.stream;
import static java.util.stream.Collectors.toList;
import static org.globalart.astrum.common.text.Strings2.MASK;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Priority.OPTIONAL;
import static org.globalart.astrum.supportzip.SupportBundle.ContentSource.Type.JMX;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Adds JMX information (mbean+readable-attributes dump) to support bundle.
 *
 * @since 3.0
 */
@Component
public class JmxCustomizer
    implements SupportBundleCustomizer
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String INFINITY = "infinity";

  private static final String NOT_A_NUMBER = "NaN";

  private final MBeanServer server;

  private final ObjectMapper objectMapper;

  private static final List<String> SENSITIVE_FIELD_NAMES =
      asList("password", "secret", "token", "sign", "auth", "cred", "key", "pass");

  @Autowired
  public JmxCustomizer(@Qualifier("platform") final MBeanServer server) {
    this.server = checkNotNull(server);
    this.objectMapper = new ObjectMapper();
  }

  @Override
  public void customize(final SupportBundle supportBundle) {
    supportBundle.add(new GeneratedContentSourceSupport(JMX, "info/jmx.json", OPTIONAL)
    {
      @Override
      protected void generate(final File file) {
        try {
          log.debug("Querying mbeans");
          Set<ObjectName> objectNames = server.queryNames(new ObjectName("*:*"), null);

          log.debug("Building model");
          Map<String, Object> model = new HashMap<>();
          for (ObjectName objectName : objectNames) {
            // normalize names, strip out quotes
            String name = objectName.getCanonicalName().replace("\"", "").replace("\'", "");
            log.debug("Processing MBean: {}", name);

            MBeanInfo info = server.getMBeanInfo(objectName);
            Map<String, Object> attrs = new HashMap<>();
            stream(info.getAttributes()).forEach(attr -> {
              log.debug("Processing MBean attribute: {}", attr);
              if (attr.isReadable() && !"ObjectName".equals(attr.getName())) {
                try {
                  Object value = server.getAttribute(objectName, attr.getName());
                  attrs.put(attr.getName(), render(value));
                }
                catch (ReflectionException | AttributeNotFoundException | InstanceNotFoundException | MBeanException
                    | RuntimeMBeanException e) {
                  log.trace("Unable to fetch attribute: {}; ignoring", attr.getName(), e);
                }
              }
            });
            model.put(name, attrs);
          }
          try (FileWriter writer = new FileWriter(file)) {
            objectMapper.writerWithDefaultPrettyPrinter().writeValue(writer, model);
          }
        }
        catch (MalformedObjectNameException | InstanceNotFoundException | IntrospectionException | ReflectionException
            | IOException e) {
          throw new RuntimeException(e);
        }
      }
    });
  }

  @VisibleForTesting
  protected Object render(final Object value) {
    if (value == null) {
      return null;
    }

    // TODO: Cope with password-like fields where we can detect .*password.* or something?

    Class<?> type = value.getClass();
    log.trace("Rendering type: {}", type);

    if (String.class.isAssignableFrom(type)) {
      return render((String) value);
    }
    if (TabularData.class.isAssignableFrom(type)) {
      return render((TabularData) value);
    }
    if (CompositeData.class.isAssignableFrom(type)) {
      return render((CompositeData) value);
    }
    if (ObjectName.class.isAssignableFrom(type)) {
      return render((ObjectName) value);
    }
    if (Collection.class.isAssignableFrom(type)) {
      return render((Collection<?>) value);
    }
    if (Object[].class.isAssignableFrom(type)) {
      return render((Object[]) value);
    }
    if (Map.class.isAssignableFrom(type)) {
      return render((Map<?, ?>) value);
    }
    if (Double.class.isAssignableFrom(type)) {
      return render((Double) value);
    }
    if (Float.class.isAssignableFrom(type)) {
      return render((Float) value);
    }
    if (Enum.class.isAssignableFrom(type)) {
      return render((Enum<?>) value);
    }
    if (isAssignableFrom(type, asList(CharSequence.class, Number.class, Boolean.class))) {
      return value;
    }
    log.trace("Coercing to String: {} -> {}", type, value);
    return String.valueOf(value);
  }

  private Object render(final String string) {
    for (String sensitiveName : SENSITIVE_FIELD_NAMES) {
      if (string.contains(sensitiveName)) {
        return string.replaceAll(sensitiveName + "=\\S*", sensitiveName + "=" + MASK);
      }
    }
    return string;
  }

  private Object render(final TabularData tabularData) {
    List<Object> result = new ArrayList<>();
    tabularData.keySet().forEach(key -> {
      CompositeData row = tabularData.get(((List<?>) key).toArray());
      result.add(render(row));
    });
    return result;
  }

  private Object render(final CompositeData compositeData) {
    Map<String, Object> result = new HashMap<>();
    compositeData.getCompositeType().keySet().forEach(key -> result.put(key, compositeData.get(key)));
    return result;
  }

  private Object render(final ObjectName objectName) {
    return objectName.getCanonicalName();
  }

  private Object render(final Collection<?> collection) {
    return collection.stream().map(this::render).collect(toList());
  }

  private Object render(final Object[] objectArray) {
    return stream(objectArray).map(this::render).collect(toList());
  }

  private Object render(final Map<?, ?> map) {
    Map<String, Object> result = new HashMap<>();
    map.forEach((k, v) -> result.put(k.toString(), render(v)));
    return result;
  }

  private Object render(final Double value) {
    if (value.isInfinite()) {
      return INFINITY;
    }
    if (value.isNaN()) {
      return NOT_A_NUMBER;
    }
    return value;
  }

  private Object render(final Float value) {
    if (value.isInfinite()) {
      return INFINITY;
    }
    if (value.isNaN()) {
      return NOT_A_NUMBER;
    }
    return value;
  }

  private Object render(final Enum<?> value) {
    return value.name();
  }

  private boolean isAssignableFrom(final Class<?> type, final List<Class<?>> checkTypes) {
    for (Class<?> checkType : checkTypes) {
      if (checkType.isAssignableFrom(type)) {
        return true;
      }
    }
    return false;
  }
}
