
package org.globalart.astrum.bootstrap.common;

import java.io.File;
import java.net.URI;
import java.time.Duration;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.cluster.ClusterCoordinationService;
import org.globalart.astrum.common.cluster.LocalClusterCoordinationService;
import org.globalart.astrum.common.conversion.BooleanPropertyEditor;
import org.globalart.astrum.common.conversion.ByteSizePropertyEditor;
import org.globalart.astrum.common.conversion.DurationPropertyEditor;
import org.globalart.astrum.common.conversion.FilePropertyEditor;
import org.globalart.astrum.common.conversion.ProviderPropertyEditor;
import org.globalart.astrum.common.conversion.TimePropertyEditor;
import org.globalart.astrum.common.conversion.URIPropertyEditor;
import org.globalart.astrum.common.io.ByteSize;
import org.globalart.astrum.common.time.Time;

import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.MapperFeature;
import com.fasterxml.jackson.databind.Module;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.fasterxml.jackson.databind.json.JsonMapper;
import com.fasterxml.jackson.databind.ser.PropertyFilter;
import com.fasterxml.jackson.databind.ser.impl.SimpleFilterProvider;
import com.fasterxml.jackson.datatype.jdk8.Jdk8Module;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import com.fasterxml.jackson.module.paramnames.ParameterNamesModule;
import jakarta.inject.Provider;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.CustomEditorConfigurer;
import org.springframework.boot.autoconfigure.condition.ConditionalOnMissingBean;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Primary;

import static com.google.common.base.Preconditions.checkNotNull;

@Configuration
public class CommonConfiguration
{
  @Bean
  public static CustomEditorConfigurer commonCustomEditorConfigurer() {
    CustomEditorConfigurer configurer = new CustomEditorConfigurer();
    configurer.setCustomEditors(Map.of(Time.class, TimePropertyEditor.class,
        ByteSize.class, ByteSizePropertyEditor.class,
        Duration.class, DurationPropertyEditor.class,
        boolean.class, BooleanPropertyEditor.class,
        URI.class, URIPropertyEditor.class,
        File.class, FilePropertyEditor.class,
        Provider.class, ProviderPropertyEditor.class));
    return configurer;
  }

  /**
   * Provider for the default configuration of {@link JsonMapper}. Marked with a sub-zero priority to prioritize
   * existing {@link ObjectMapper} providers.<br/>
   */
  @Qualifier("default")
  @Bean
  @Primary
  public static JsonMapper jsonMapper(final List<PropertyFilter> filters, final List<Module> modules) {
    return JsonMapper.builder()
        .enable(MapperFeature.ACCEPT_CASE_INSENSITIVE_ENUMS)
        .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
        .addModule(new JavaTimeModule())
        .addModule(new Jdk8Module())
        .addModule(new ParameterNamesModule())
        .addModules(modules)
        .disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS)
        .filterProvider(new SimpleFilterProvider(QualifierUtil.buildQualifierBeanMap(checkNotNull(filters)))
            .setFailOnUnknownId(false))
        .build();
  }

  @Bean
  @ConditionalOnMissingBean(ClusterCoordinationService.class)
  public ClusterCoordinationService localClusterCoordinationService() {
    return new LocalClusterCoordinationService();
  }
}
