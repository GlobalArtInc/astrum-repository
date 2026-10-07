
package org.globalart.astrum.metrics;

import com.codahale.metrics.Meter;
import com.codahale.metrics.MetricRegistry;
import com.codahale.metrics.SharedMetricRegistries;
import com.codahale.metrics.Timer;
import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;
import org.mockito.MockedStatic;
import org.springframework.beans.factory.config.BeanDefinition;
import org.springframework.beans.factory.support.BeanDefinitionRegistry;

import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.mockStatic;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.common.metrics.MetricsConstants.ASTRUM_METRICS_REGISTRY_NAME;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

@ExtendWith(MockitoExtension.class)
public class MetricAnnotationsMethodPostProcessorTest
{
  MockedStatic<SharedMetricRegistries> sharedMetricRegistries = mockStatic(SharedMetricRegistries.class);

  @Mock
  private MetricRegistry metricRegistry;

  private MetricAnnotationsMethodPostProcessor postProcessor;

  @BeforeEach
  public void setUp() {
    sharedMetricRegistries.when(() -> SharedMetricRegistries.getOrCreate(ASTRUM_METRICS_REGISTRY_NAME))
        .thenReturn(metricRegistry);

    postProcessor = new MetricAnnotationsMethodPostProcessor();
  }

  @Test
  void testPostProcessBeanDefinitionRegistry() {
    BeanDefinitionRegistry registry = mock(BeanDefinitionRegistry.class);
    BeanDefinition timedDef = mock(BeanDefinition.class);
    BeanDefinition meteredDef = mock(BeanDefinition.class);
    BeanDefinition mixedDef = mock(BeanDefinition.class);
    BeanDefinition noAnnoDef = mock(BeanDefinition.class);

    String[] beanNames = {"timedBean", "meteredBean", "mixedBean", "noAnnoBean"};
    when(registry.getBeanDefinitionNames()).thenReturn(beanNames);
    when(registry.getBeanDefinition("timedBean")).thenReturn(timedDef);
    when(registry.getBeanDefinition("meteredBean")).thenReturn(meteredDef);
    when(registry.getBeanDefinition("mixedBean")).thenReturn(mixedDef);
    when(registry.getBeanDefinition("noAnnoBean")).thenReturn(noAnnoDef);
    when(timedDef.getBeanClassName()).thenReturn(TimedClass.class.getName());
    when(meteredDef.getBeanClassName()).thenReturn(MeteredClass.class.getName());
    when(mixedDef.getBeanClassName()).thenReturn(MixedClass.class.getName());
    when(noAnnoDef.getBeanClassName()).thenReturn(NoAnnoClass.class.getName());

    Timer timer = mock(Timer.class);
    Meter meter = mock(Meter.class);
    when(metricRegistry.timer(anyString())).thenReturn(timer);
    when(metricRegistry.meter(anyString())).thenReturn(meter);

    postProcessor.postProcessBeanDefinitionRegistry(registry);

    verify(metricRegistry).timer(
        "org.globalart.astrum.metrics.MetricAnnotationsMethodPostProcessorTest$TimedClass.custom.timer.name");
    verify(metricRegistry).meter(
        "org.globalart.astrum.metrics.MetricAnnotationsMethodPostProcessorTest$MeteredClass.custom.meter.name");
    verify(metricRegistry).timer(
        "org.globalart.astrum.metrics.MetricAnnotationsMethodPostProcessorTest$MixedClass.both.timer");
    verify(metricRegistry).meter(
        "org.globalart.astrum.metrics.MetricAnnotationsMethodPostProcessorTest$MixedClass.both.exceptions");
    verify(metricRegistry, never())
        .timer(
            "org.globalart.astrum.metrics.MetricAnnotationsMethodPostProcessorTest$NoAnnoClass.plain.timer");
    verify(metricRegistry, never()).meter(
        "org.globalart.astrum.metrics.MetricAnnotationsMethodPostProcessorTest$NoAnnoClass.plain.exceptions");
  }

  @AfterEach
  public void tearDown() {
    sharedMetricRegistries.close();
  }

  static class TimedClass
  {
    @Timed(name = "custom.timer.name")
    public void timedMethod() {
    }
  }

  static class MeteredClass
  {
    @ExceptionMetered(name = "custom.meter.name")
    public void meteredMethod() {
    }
  }

  static class MixedClass
  {
    @Timed
    @ExceptionMetered
    public void both() {
    }
  }

  static class NoAnnoClass
  {
    public void plain() {
    }
  }
}
