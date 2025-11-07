
package org.globalart.astrum.testcommon.extensions;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;
import java.lang.reflect.Field;
import java.util.List;
import java.util.UUID;
import java.util.stream.Stream;

import ch.qos.logback.classic.Logger;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.read.ListAppender;
import org.junit.jupiter.api.extension.AfterEachCallback;
import org.junit.jupiter.api.extension.BeforeEachCallback;
import org.junit.jupiter.api.extension.Extension;
import org.junit.jupiter.api.extension.ExtensionContext;
import org.junit.platform.commons.support.AnnotationSupport;
import org.slf4j.LoggerFactory;
import org.slf4j.event.Level;

/**
 * A junit5 extension to capture logging while a test is running
 *
 * <pre>
 * &#64;LogFor(MyClass.class)
 * TestLogAccessor logs;
 *
 * &#64;LogFor(value = AnotherClass.class, level = Level.DEBUG)
 * TestLogAccessor logs;
 * </pre>
 *
 * @see org.globalart.astrum.testcommon.matchers.AstrumMatchers#logMessage
 * @see org.globalart.astrum.testcommon.matchers.AstrumMatchers#logLevel
 */
public class LoggingExtension
    implements Extension, BeforeEachCallback, AfterEachCallback
{
  /**
   * Annotation for {@link TestLogAccessor}
   */
  @Target({ElementType.FIELD})
  @Retention(RetentionPolicy.RUNTIME)
  public @interface CaptureLogsFor
  {
    /**
     * The class of the logger
     */
    Class<?> value();

    /**
     * Set the logging level for the test
     */
    Level level() default Level.INFO;
  }

  /**
   * An accessor for logs during a test
   */
  public interface TestLogAccessor
  {
    /**
     * A list of the {@link ILoggingEvent} which occurred during this test run
     * 
     * @return
     */
    List<ILoggingEvent> logs();
  }

  @Override
  public void beforeEach(final ExtensionContext context) throws Exception {
    context.getTestClass()
        .map(this::find)
        .orElseGet(Stream::of)
        .forEach(field -> setLogger(context.getRequiredTestInstance(), field));
  }

  @Override
  public void afterEach(final ExtensionContext context) throws Exception {
    context.getTestClass()
        .map(this::find)
        .orElseGet(Stream::of)
        .forEach(field -> unsetLogger(context.getRequiredTestInstance(), field));
  }

  /*
   * Set the Logs field in the test and update the logging level
   */
  private static void setLogger(final Object test, final Field field) {
    LogsImpl logs = new LogsImpl();
    Logger logger = getLogger(field);
    logger.setLevel(convert(field.getAnnotation(CaptureLogsFor.class).level()));
    logger.addAppender(logs);
    try {
      field.setAccessible(true);
      field.set(test, logs);
    }
    catch (Exception e) {
      if (e instanceof RuntimeException re) {
        throw re;
      }
      throw new RuntimeException(e);
    }
  }

  /*
   * Unset the logger in logback after a test
   */
  private static void unsetLogger(final Object test, final Field field) {
    try {
      field.setAccessible(true);
      Logger logger = getLogger(field);
      logger.detachAppender((LogsImpl) field.get(test));
      logger.setLevel(convert(Level.INFO));
    }
    catch (Exception e) {
      if (e instanceof RuntimeException re) {
        throw re;
      }
      throw new RuntimeException(e);
    }
  }

  /*
   * Retrieve the logback Logger for the field annotated with LogFor
   */
  private static Logger getLogger(final Field field) {
    CaptureLogsFor logFor = field.getAnnotation(CaptureLogsFor.class);

    return (Logger) LoggerFactory.getLogger(logFor.value());
  }

  /*
   * Find fields annotated with LogFor in the test class
   */
  private Stream<Field> find(final Class<?> testClass) {
    return AnnotationSupport.findAnnotatedFields(testClass, CaptureLogsFor.class)
        .stream();
  }

  private static class LogsImpl
      extends ListAppender<ILoggingEvent>
      implements TestLogAccessor
  {
    LogsImpl() {
      setName(UUID.randomUUID().toString());
      start();
    }

    @Override
    public List<ILoggingEvent> logs() {
      return List.copyOf(list);
    }
  }

  /**
   * Utility to switch between slf4j logger level and Logbacks internal level class
   */
  public static ch.qos.logback.classic.Level convert(final Level level) {
    return switch (level) {
      case DEBUG -> ch.qos.logback.classic.Level.DEBUG;
      case ERROR -> ch.qos.logback.classic.Level.ERROR;
      case INFO -> ch.qos.logback.classic.Level.INFO;
      case TRACE -> ch.qos.logback.classic.Level.TRACE;
      case WARN -> ch.qos.logback.classic.Level.WARN;
    };
  }
}
