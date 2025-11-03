
package org.globalart.astrum.common.metrics;

import java.lang.reflect.Method;

import com.codahale.metrics.Meter;
import com.codahale.metrics.MetricRegistry;
import com.codahale.metrics.SharedMetricRegistries;
import com.codahale.metrics.annotation.ExceptionMetered;
import org.aspectj.lang.ProceedingJoinPoint;
import org.aspectj.lang.reflect.MethodSignature;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.MockedStatic;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mockStatic;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.common.metrics.MetricsConstants.ASTRUM_METRICS_REGISTRY_NAME;

@ExtendWith(MockitoExtension.class)
public class ExceptionMeteredAspectTest
{
  MockedStatic<SharedMetricRegistries> sharedMetricRegistries = mockStatic(SharedMetricRegistries.class);

  @Mock
  private MetricRegistry metricRegistry;

  @Mock
  private ProceedingJoinPoint joinPoint;

  @Mock
  private MethodSignature methodSignature;

  @Mock
  private Meter meter;

  private ExceptionMeteredAspect exceptionMeteredAspect;

  @BeforeEach
  public void setUp() {
    sharedMetricRegistries.when(() -> SharedMetricRegistries.getOrCreate(ASTRUM_METRICS_REGISTRY_NAME))
        .thenReturn(metricRegistry);

    exceptionMeteredAspect = new ExceptionMeteredAspect();
  }

  @Test
  void testAroundItShouldMeterWithAnnoName() throws Throwable {
    Method method = TestClass.class.getMethod("testMethodWithAbsoluteName");
    ExceptionMetered exceptionMetered = method.getAnnotation(ExceptionMetered.class);
    when(joinPoint.getSignature()).thenReturn(methodSignature);
    when(methodSignature.getMethod()).thenReturn(method);
    when(joinPoint.proceed()).thenThrow(new RuntimeException("Test exception"));
    when(metricRegistry.meter(anyString())).thenReturn(meter);

    assertThrows(RuntimeException.class, () -> exceptionMeteredAspect.around(joinPoint, exceptionMetered));

    verify(metricRegistry).meter("testMetric");
    verify(meter).mark();
  }

  @Test
  void testAroundItShouldMeterWithMethodName() throws Throwable {
    Method method = TestClass.class.getMethod("testMethodNamed");
    ExceptionMetered exceptionMetered = method.getAnnotation(ExceptionMetered.class);
    when(joinPoint.getSignature()).thenReturn(methodSignature);
    when(methodSignature.getMethod()).thenReturn(method);
    when(joinPoint.proceed()).thenThrow(new IllegalArgumentException("Test exception"));
    when(metricRegistry.meter(anyString())).thenReturn(meter);

    assertThrows(IllegalArgumentException.class, () -> exceptionMeteredAspect.around(joinPoint, exceptionMetered));

    verify(metricRegistry)
        .meter("org.globalart.astrum.common.metrics.ExceptionMeteredAspectTest$TestClass.testMethodNamed.exceptions");
    verify(meter).mark();
  }

  @Test
  void testAroundWhenThrowsDifferentException() throws Throwable {
    Method method = TestClass.class.getMethod("testMethodNamed");
    ExceptionMetered exceptionMetered = method.getAnnotation(ExceptionMetered.class);
    when(joinPoint.getSignature()).thenReturn(methodSignature);
    when(methodSignature.getMethod()).thenReturn(method);
    when(joinPoint.proceed()).thenThrow(new RuntimeException("Test exception"));

    assertThrows(RuntimeException.class, () -> exceptionMeteredAspect.around(joinPoint, exceptionMetered));

    verify(metricRegistry, never()).meter("testMetric");
    verify(meter, never()).mark();
  }

  @AfterEach
  public void tearDown() {
    sharedMetricRegistries.close();
  }

  private static class TestClass
  {
    @ExceptionMetered(name = "testMetric", absolute = true)
    public void testMethodWithAbsoluteName() {
      // no-op
    }

    @ExceptionMetered(cause = IllegalArgumentException.class)
    public void testMethodNamed() {
      // no-op
    }
  }
}
